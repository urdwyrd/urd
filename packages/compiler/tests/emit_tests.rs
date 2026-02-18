// Tests for Phase 5: EMIT
//
// Test categories from the EMIT phase brief:
// 1. World block (4)
// 2. Type block (6)
// 3. Entity block (5)
// 4. Location block (6)
// 5. Condition lowering (8)
// 6. Effect lowering (8)
// 7. Sequence and advance (5)
// 8. Dialogue block (14)
// 9. Determinism (4)
// 10. Integration (4)

use indexmap::IndexMap;
use urd_compiler::ast::*;
use urd_compiler::diagnostics::DiagnosticCollector;
use urd_compiler::graph::{CompilationUnit, DependencyGraph, FileNode};
use urd_compiler::link;
use urd_compiler::emit;
use urd_compiler::validate;
use urd_compiler::span::Span;

// ── Helpers ──

fn span(file: &str, line: u32) -> Span {
    Span::new(file.to_string(), line, 1, line, 80)
}

fn make_file_ast(path: &str, frontmatter: Option<Frontmatter>, content: Vec<ContentNode>) -> FileAst {
    FileAst {
        path: path.to_string(),
        frontmatter,
        content,
        span: span(path, 1),
    }
}

fn make_frontmatter(entries: Vec<FrontmatterEntry>) -> Frontmatter {
    Frontmatter {
        entries,
        span: span("test.urd.md", 1),
    }
}

fn fm_entry(key: &str, value: FrontmatterValue) -> FrontmatterEntry {
    FrontmatterEntry {
        key: key.to_string(),
        value,
        span: span("test.urd.md", 1),
    }
}

fn make_type_def(name: &str, traits: Vec<&str>, properties: Vec<PropertyDef>) -> FrontmatterValue {
    FrontmatterValue::TypeDef(TypeDef {
        name: name.to_string(),
        traits: traits.into_iter().map(String::from).collect(),
        properties,
        span: span("test.urd.md", 2),
    })
}

fn make_property(name: &str, prop_type: &str) -> PropertyDef {
    PropertyDef {
        name: name.to_string(),
        property_type: prop_type.to_string(),
        default: None,
        visibility: None,
        values: None,
        min: None,
        max: None,
        ref_type: None,
        element_type: None,
        element_values: None,
        element_ref_type: None,
        description: None,
        span: span("test.urd.md", 3),
    }
}

fn make_property_with_default(name: &str, prop_type: &str, default: Scalar) -> PropertyDef {
    PropertyDef {
        default: Some(default),
        ..make_property(name, prop_type)
    }
}

fn make_property_with_values(name: &str, prop_type: &str, values: Vec<&str>) -> PropertyDef {
    PropertyDef {
        values: Some(values.into_iter().map(String::from).collect()),
        ..make_property(name, prop_type)
    }
}

#[allow(dead_code)]
fn make_property_with_range(name: &str, prop_type: &str, min: Option<f64>, max: Option<f64>) -> PropertyDef {
    PropertyDef {
        min,
        max,
        ..make_property(name, prop_type)
    }
}

fn make_property_with_ref_type(name: &str, ref_type: &str) -> PropertyDef {
    PropertyDef {
        property_type: "ref".to_string(),
        ref_type: Some(ref_type.to_string()),
        ..make_property(name, "ref")
    }
}

fn make_property_hidden(name: &str, prop_type: &str) -> PropertyDef {
    PropertyDef {
        visibility: Some("hidden".to_string()),
        ..make_property(name, prop_type)
    }
}

fn make_entity_decl(id: &str, type_name: &str, overrides: Vec<(&str, Scalar)>) -> FrontmatterValue {
    FrontmatterValue::EntityDecl(EntityDecl {
        id: id.to_string(),
        type_name: type_name.to_string(),
        property_overrides: overrides.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
        annotation: None,
        span: span("test.urd.md", 4),
    })
}

fn make_world_block(fields: Vec<(&str, Scalar)>) -> FrontmatterValue {
    FrontmatterValue::WorldBlock(WorldBlock {
        fields: fields.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
        span: span("test.urd.md", 5),
    })
}

fn location(name: &str) -> ContentNode {
    ContentNode::LocationHeading(LocationHeading {
        display_name: name.to_string(),
        span: span("test.urd.md", 10),
    })
}

fn section(name: &str) -> ContentNode {
    ContentNode::SectionLabel(SectionLabel {
        name: name.to_string(),
        span: span("test.urd.md", 15),
    })
}

fn prose(text: &str) -> ContentNode {
    ContentNode::Prose(Prose {
        text: text.to_string(),
        span: span("test.urd.md", 11),
    })
}

fn choice(label: &str, sticky: bool) -> ContentNode {
    ContentNode::Choice(Choice {
        sticky,
        label: label.to_string(),
        target: None,
        target_type: None,
        content: Vec::new(),
        indent_level: 1,
        annotation: None,
        span: span("test.urd.md", 20),
    })
}

fn choice_with_content(label: &str, sticky: bool, content: Vec<ContentNode>) -> ContentNode {
    ContentNode::Choice(Choice {
        sticky,
        label: label.to_string(),
        target: None,
        target_type: None,
        content,
        indent_level: 1,
        annotation: None,
        span: span("test.urd.md", 20),
    })
}

fn choice_with_target(label: &str, target: &str) -> ContentNode {
    ContentNode::Choice(Choice {
        sticky: false,
        label: label.to_string(),
        target: Some(target.to_string()),
        target_type: None,
        content: Vec::new(),
        indent_level: 1,
        annotation: None,
        span: span("test.urd.md", 20),
    })
}

fn choice_with_target_type(label: &str, target_type: &str) -> ContentNode {
    ContentNode::Choice(Choice {
        sticky: false,
        label: label.to_string(),
        target: None,
        target_type: Some(target_type.to_string()),
        content: Vec::new(),
        indent_level: 1,
        annotation: None,
        span: span("test.urd.md", 20),
    })
}

fn property_comparison(entity_ref: &str, property: &str, op: &str, value: &str) -> ContentNode {
    ContentNode::Condition(Condition {
        expr: ConditionExpr::PropertyComparison(PropertyComparison {
            entity_ref: entity_ref.to_string(),
            property: property.to_string(),
            operator: op.to_string(),
            value: value.to_string(),
            annotation: None,
            span: span("test.urd.md", 40),
        }),
        indent_level: 0,
        span: span("test.urd.md", 40),
    })
}

fn containment_check(entity_ref: &str, container_ref: &str, negated: bool) -> ContentNode {
    ContentNode::Condition(Condition {
        expr: ConditionExpr::ContainmentCheck(ContainmentCheck {
            entity_ref: entity_ref.to_string(),
            container_ref: container_ref.to_string(),
            negated,
            annotation: None,
            span: span("test.urd.md", 41),
        }),
        indent_level: 0,
        span: span("test.urd.md", 41),
    })
}

fn exhaustion_check(section_name: &str) -> ContentNode {
    ContentNode::Condition(Condition {
        expr: ConditionExpr::ExhaustionCheck(ExhaustionCheck {
            section_name: section_name.to_string(),
            annotation: None,
            span: span("test.urd.md", 42),
        }),
        indent_level: 0,
        span: span("test.urd.md", 42),
    })
}

fn set_effect(target_prop: &str, value: &str) -> ContentNode {
    ContentNode::Effect(Effect {
        effect_type: EffectType::Set {
            target_prop: target_prop.to_string(),
            operator: "=".to_string(),
            value_expr: value.to_string(),
        },
        indent_level: 0,
        annotation: None,
        span: span("test.urd.md", 45),
    })
}

fn set_effect_arithmetic(target_prop: &str, operator: &str, value: &str) -> ContentNode {
    ContentNode::Effect(Effect {
        effect_type: EffectType::Set {
            target_prop: target_prop.to_string(),
            operator: operator.to_string(),
            value_expr: value.to_string(),
        },
        indent_level: 0,
        annotation: None,
        span: span("test.urd.md", 45),
    })
}

fn move_effect(entity_ref: &str, destination_ref: &str) -> ContentNode {
    ContentNode::Effect(Effect {
        effect_type: EffectType::Move {
            entity_ref: entity_ref.to_string(),
            destination_ref: destination_ref.to_string(),
        },
        indent_level: 0,
        annotation: None,
        span: span("test.urd.md", 46),
    })
}

fn reveal_effect(target_prop: &str) -> ContentNode {
    ContentNode::Effect(Effect {
        effect_type: EffectType::Reveal {
            target_prop: target_prop.to_string(),
        },
        indent_level: 0,
        annotation: None,
        span: span("test.urd.md", 47),
    })
}

fn destroy_effect(entity_ref: &str) -> ContentNode {
    ContentNode::Effect(Effect {
        effect_type: EffectType::Destroy {
            entity_ref: entity_ref.to_string(),
        },
        indent_level: 0,
        annotation: None,
        span: span("test.urd.md", 48),
    })
}

fn entity_speech(entity_ref: &str, text: &str) -> ContentNode {
    ContentNode::EntitySpeech(EntitySpeech {
        entity_ref: entity_ref.to_string(),
        text: text.to_string(),
        annotation: None,
        span: span("test.urd.md", 30),
    })
}

fn entity_presence(refs: Vec<&str>) -> ContentNode {
    let len = refs.len();
    ContentNode::EntityPresence(EntityPresence {
        entity_refs: refs.into_iter().map(String::from).collect(),
        annotations: vec![None; len],
        span: span("test.urd.md", 12),
    })
}

fn exit_decl(direction: &str, destination: &str) -> ContentNode {
    ContentNode::ExitDeclaration(ExitDeclaration {
        direction: direction.to_string(),
        destination: destination.to_string(),
        children: Vec::new(),
        annotation: None,
        span: span("test.urd.md", 25),
    })
}

fn exit_decl_with_children(direction: &str, destination: &str, children: Vec<ContentNode>) -> ContentNode {
    ContentNode::ExitDeclaration(ExitDeclaration {
        direction: direction.to_string(),
        destination: destination.to_string(),
        children,
        annotation: None,
        span: span("test.urd.md", 25),
    })
}

fn blocked_message(text: &str) -> ContentNode {
    ContentNode::BlockedMessage(BlockedMessage {
        text: text.to_string(),
        indent_level: 1,
        span: span("test.urd.md", 26),
    })
}

fn jump(target: &str) -> ContentNode {
    ContentNode::Jump(Jump {
        target: target.to_string(),
        is_exit_qualified: false,
        indent_level: 0,
        annotation: None,
        span: span("test.urd.md", 35),
    })
}

fn sequence_heading(name: &str) -> ContentNode {
    ContentNode::SequenceHeading(SequenceHeading {
        display_name: name.to_string(),
        span: span("test.urd.md", 55),
    })
}

fn phase_heading(name: &str, auto: bool) -> ContentNode {
    ContentNode::PhaseHeading(PhaseHeading {
        display_name: name.to_string(),
        auto,
        span: span("test.urd.md", 56),
    })
}

fn single_file_cu(ast: FileAst) -> CompilationUnit {
    let path = ast.path.clone();
    let mut nodes = IndexMap::new();
    nodes.insert(
        path.clone(),
        FileNode {
            path: path.clone(),
            ast,
            imports: Vec::new(),
        },
    );
    CompilationUnit {
        graph: DependencyGraph {
            nodes,
            edges: Vec::new(),
            entry_path: Some(path.clone()),
        },
        ordered_asts: vec![path],
    }
}

fn two_file_cu(ast_b: FileAst, ast_a: FileAst) -> CompilationUnit {
    let path_a = ast_a.path.clone();
    let path_b = ast_b.path.clone();
    let mut nodes = IndexMap::new();
    nodes.insert(
        path_b.clone(),
        FileNode {
            path: path_b.clone(),
            ast: ast_b,
            imports: Vec::new(),
        },
    );
    nodes.insert(
        path_a.clone(),
        FileNode {
            path: path_a.clone(),
            ast: ast_a,
            imports: vec![path_b.clone()],
        },
    );
    CompilationUnit {
        graph: DependencyGraph {
            nodes,
            edges: vec![(path_a.clone(), path_b.clone())],
            entry_path: Some(path_a.clone()),
        },
        ordered_asts: vec![path_b, path_a],
    }
}

/// Link + validate + emit, returning JSON string and diagnostics.
fn link_and_emit(cu: CompilationUnit) -> (String, DiagnosticCollector) {
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);
    validate::validate(&linked.graph, &linked.symbol_table, &mut diag);
    assert!(
        !diag.has_errors(),
        "Errors before EMIT: {:?}",
        diag.all().iter().map(|d| &d.message).collect::<Vec<_>>()
    );
    let json = emit::emit(&linked.graph, &linked.symbol_table, &mut diag);
    (json, diag)
}

/// Link + validate + emit, returning parsed JSON.
fn emit_json(cu: CompilationUnit) -> serde_json::Value {
    let (json, _) = link_and_emit(cu);
    serde_json::from_str(&json).expect("EMIT output should be valid JSON")
}

/// Link, modify the symbol table, then emit.
fn link_modify_and_emit<F>(cu: CompilationUnit, modify: F) -> (String, DiagnosticCollector)
where
    F: FnOnce(&mut urd_compiler::symbol_table::SymbolTable),
{
    let mut diag = DiagnosticCollector::new();
    let mut linked = link::link(cu, &mut diag);
    modify(&mut linked.symbol_table);
    let json = emit::emit(&linked.graph, &linked.symbol_table, &mut diag);
    (json, diag)
}

/// Build a minimal world CU with one location.
fn minimal_world_cu() -> CompilationUnit {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![location("Cell")],
    );
    single_file_cu(ast)
}

// ── World Block Tests ──

#[test]
fn world_minimal() {
    let json = emit_json(minimal_world_cu());
    let world = &json["world"];
    assert_eq!(world["name"], "test");
    assert_eq!(world["urd"], "1");
    assert_eq!(world["start"], "cell");
    assert!(world.get("version").is_none());
    assert!(world.get("description").is_none());
    assert!(world.get("author").is_none());
    assert!(world.get("entry").is_none());
    assert!(world.get("seed").is_none());
}

#[test]
fn world_full_metadata() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("puzzle".to_string())),
                ("version", Scalar::String("1.0".to_string())),
                ("description", Scalar::String("A test puzzle.".to_string())),
                ("author", Scalar::String("Test Author".to_string())),
                ("start", Scalar::String("Cell".to_string())),
                ("seed", Scalar::Integer(42)),
            ])),
        ])),
        vec![location("Cell")],
    );
    let json = emit_json(single_file_cu(ast));
    let world = &json["world"];
    assert_eq!(world["name"], "puzzle");
    assert_eq!(world["urd"], "1");
    assert_eq!(world["version"], "1.0");
    assert_eq!(world["description"], "A test puzzle.");
    assert_eq!(world["author"], "Test Author");
    assert_eq!(world["start"], "cell");
    assert_eq!(world["seed"], 42);
}

#[test]
fn world_urd_override() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("urd", Scalar::String("2".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![location("Cell")],
    );
    let (json_str, _) = link_and_emit(single_file_cu(ast));
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(json["world"]["urd"], "1");
}

#[test]
fn world_absent_optional_fields() {
    let json = emit_json(minimal_world_cu());
    let world = &json["world"];
    let obj = world.as_object().unwrap();
    assert!(obj.contains_key("name"));
    assert!(obj.contains_key("urd"));
    assert!(obj.contains_key("start"));
    assert!(!obj.contains_key("version"));
    assert!(!obj.contains_key("description"));
    assert!(!obj.contains_key("author"));
    assert!(!obj.contains_key("entry"));
    assert!(!obj.contains_key("seed"));
}

// ── Type Block Tests ──

#[test]
fn type_all_property_types() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Item", make_type_def("Item", vec![], vec![
                make_property("active", "boolean"),
                make_property("count", "integer"),
                make_property("weight", "number"),
                make_property("label", "string"),
                make_property_with_values("mood", "enum", vec!["happy", "sad"]),
                make_property_with_ref_type("owner", "Item"),
            ])),
        ])),
        vec![location("Cell")],
    );
    let json = emit_json(single_file_cu(ast));
    let props = &json["types"]["Item"]["properties"];
    assert_eq!(props["active"]["type"], "boolean");
    assert_eq!(props["count"]["type"], "integer");
    assert_eq!(props["weight"]["type"], "number");
    assert_eq!(props["label"]["type"], "string");
    assert_eq!(props["mood"]["type"], "enum");
    assert_eq!(props["mood"]["values"][0], "happy");
    assert_eq!(props["mood"]["values"][1], "sad");
    assert_eq!(props["owner"]["type"], "ref");
    assert_eq!(props["owner"]["ref_type"], "Item");
}

#[test]
fn type_hidden_property() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Item", make_type_def("Item", vec![], vec![
                make_property_hidden("secret", "string"),
            ])),
        ])),
        vec![location("Cell")],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["types"]["Item"]["properties"]["secret"]["visibility"], "hidden");
}

#[test]
fn type_visible_property_default() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Item", make_type_def("Item", vec![], vec![
                make_property("name", "string"),
            ])),
        ])),
        vec![location("Cell")],
    );
    let json = emit_json(single_file_cu(ast));
    assert!(json["types"]["Item"]["properties"]["name"].get("visibility").is_none());
}

#[test]
fn type_with_traits() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Key", make_type_def("Key", vec!["portable"], vec![
                make_property("name", "string"),
            ])),
        ])),
        vec![location("Cell")],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["types"]["Key"]["traits"][0], "portable");
}

#[test]
fn type_with_no_traits() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Door", make_type_def("Door", vec![], vec![
                make_property("locked", "boolean"),
            ])),
        ])),
        vec![location("Cell")],
    );
    let json = emit_json(single_file_cu(ast));
    assert!(json["types"]["Door"].get("traits").is_none());
}

#[test]
fn omit_empty_types_block() {
    let json = emit_json(minimal_world_cu());
    assert!(json.get("types").is_none());
}

// ── Entity Block Tests ──

#[test]
fn entity_with_overrides() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Key", make_type_def("Key", vec!["portable"], vec![
                make_property("name", "string"),
            ])),
            fm_entry("rusty_key", make_entity_decl("rusty_key", "Key", vec![
                ("name", Scalar::String("Rusty Key".to_string())),
            ])),
        ])),
        vec![location("Cell")],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["entities"]["rusty_key"]["type"], "Key");
    assert_eq!(json["entities"]["rusty_key"]["properties"]["name"], "Rusty Key");
}

#[test]
fn entity_no_overrides() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Door", make_type_def("Door", vec![], vec![
                make_property("locked", "boolean"),
            ])),
            fm_entry("cell_door", make_entity_decl("cell_door", "Door", vec![])),
        ])),
        vec![location("Cell")],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["entities"]["cell_door"]["type"], "Door");
    assert!(json["entities"]["cell_door"].get("properties").is_none());
}

#[test]
fn no_explicit_player() {
    let json = emit_json(minimal_world_cu());
    assert!(json.get("entities").is_none());
}

#[test]
fn explicit_player() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Hero", make_type_def("Hero", vec!["mobile", "container"], vec![
                make_property("health", "integer"),
            ])),
            fm_entry("player", make_entity_decl("player", "Hero", vec![
                ("health", Scalar::Integer(100)),
            ])),
        ])),
        vec![location("Cell")],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["entities"]["player"]["type"], "Hero");
    assert_eq!(json["entities"]["player"]["properties"]["health"], 100);
}

#[test]
fn entity_ref_override() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Key", make_type_def("Key", vec!["portable"], vec![
                make_property("name", "string"),
            ])),
            fm_entry("Door", make_type_def("Door", vec![], vec![
                make_property_with_ref_type("requires", "Key"),
            ])),
            fm_entry("rusty_key", make_entity_decl("rusty_key", "Key", vec![])),
            fm_entry("door", make_entity_decl("door", "Door", vec![
                ("requires", Scalar::String("rusty_key".to_string())),
            ])),
        ])),
        vec![location("Cell")],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["entities"]["door"]["properties"]["requires"], "rusty_key");
}

// ── Location Block Tests ──

#[test]
fn location_with_description() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![location("Cell"), prose("A dim stone cell.")],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["locations"]["cell"]["description"], "A dim stone cell.");
}

#[test]
fn location_with_contains() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Key", make_type_def("Key", vec!["portable"], vec![])),
            fm_entry("rusty_key", make_entity_decl("rusty_key", "Key", vec![])),
        ])),
        vec![location("Cell"), entity_presence(vec!["rusty_key"])],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["locations"]["cell"]["contains"][0], "rusty_key");
}

#[test]
fn location_with_exits() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![location("Cell"), exit_decl("north", "Corridor"), location("Corridor")],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["locations"]["cell"]["exits"]["north"]["to"], "corridor");
}

#[test]
fn exit_with_condition() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Door", make_type_def("Door", vec![], vec![
                make_property_with_default("locked", "boolean", Scalar::Boolean(true)),
            ])),
            fm_entry("door", make_entity_decl("door", "Door", vec![])),
        ])),
        vec![
            location("Cell"),
            exit_decl_with_children("north", "Corridor", vec![
                property_comparison("door", "locked", "==", "false"),
            ]),
            location("Corridor"),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["locations"]["cell"]["exits"]["north"]["condition"], "door.locked == false");
}

#[test]
fn exit_with_blocked_message() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![
            location("Cell"),
            exit_decl_with_children("north", "Corridor", vec![
                blocked_message("The door is locked."),
            ]),
            location("Corridor"),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["locations"]["cell"]["exits"]["north"]["blocked_message"], "The door is locked.");
}

#[test]
fn omit_empty_locations_block() {
    let json = emit_json(minimal_world_cu());
    assert!(json.get("locations").is_some()); // Minimal world always has a location.
}

// ── Condition Lowering Tests ──

#[test]
fn condition_property_comparison() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Guard", make_type_def("Guard", vec![], vec![
                make_property_with_values("mood", "enum", vec!["neutral", "helpful"]),
            ])),
            fm_entry("guard", make_entity_decl("guard", "Guard", vec![])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            property_comparison("guard", "mood", "==", "neutral"),
            choice("Ask", false),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["dialogue"]["test/topics"]["conditions"][0], "guard.mood == neutral");
}

#[test]
fn condition_containment_entity() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Box", make_type_def("Box", vec!["container"], vec![])),
            fm_entry("Key", make_type_def("Key", vec!["portable"], vec![])),
            fm_entry("chest", make_entity_decl("chest", "Box", vec![])),
            fm_entry("key", make_entity_decl("key", "Key", vec![])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            containment_check("key", "chest", false),
            choice("Look", false),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["dialogue"]["test/topics"]["conditions"][0], "key.container == chest");
}

#[test]
fn condition_containment_here() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Key", make_type_def("Key", vec!["portable"], vec![])),
            fm_entry("key", make_entity_decl("key", "Key", vec![])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            containment_check("key", "here", false),
            choice("Look", false),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["dialogue"]["test/topics"]["conditions"][0], "key.container == player.container");
}

#[test]
fn condition_containment_player() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Key", make_type_def("Key", vec!["portable"], vec![])),
            fm_entry("key", make_entity_decl("key", "Key", vec![])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            containment_check("key", "player", false),
            choice("Look", false),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["dialogue"]["test/topics"]["conditions"][0], "key.container == player");
}

#[test]
fn condition_containment_negated() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Key", make_type_def("Key", vec!["portable"], vec![])),
            fm_entry("key", make_entity_decl("key", "Key", vec![])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            containment_check("key", "player", true),
            choice("Look", false),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["dialogue"]["test/topics"]["conditions"][0], "key.container != player");
}

#[test]
fn condition_exhaustion() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            choice("Ask", false),
            section("farewell"),
            exhaustion_check("topics"),
            choice("Bye", false),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["dialogue"]["test/farewell"]["conditions"][0], "test/topics.exhausted");
}

#[test]
fn condition_and() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Guard", make_type_def("Guard", vec![], vec![
                make_property_with_values("mood", "enum", vec!["neutral", "helpful"]),
                make_property("trust", "integer"),
            ])),
            fm_entry("guard", make_entity_decl("guard", "Guard", vec![])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            property_comparison("guard", "mood", "==", "neutral"),
            property_comparison("guard", "trust", ">", "10"),
            choice("Ask", false),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    let conds = &json["dialogue"]["test/topics"]["conditions"];
    assert!(conds.is_array());
    assert_eq!(conds.as_array().unwrap().len(), 2);
}

#[test]
fn condition_or() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Guard", make_type_def("Guard", vec![], vec![
                make_property_with_values("mood", "enum", vec!["neutral", "helpful"]),
                make_property("trust", "integer"),
            ])),
            fm_entry("guard", make_entity_decl("guard", "Guard", vec![])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            ContentNode::OrConditionBlock(OrConditionBlock {
                conditions: vec![
                    ConditionExpr::PropertyComparison(PropertyComparison {
                        entity_ref: "guard".to_string(),
                        property: "mood".to_string(),
                        operator: "==".to_string(),
                        value: "helpful".to_string(),
                        annotation: None,
                        span: span("test.urd.md", 40),
                    }),
                    ConditionExpr::PropertyComparison(PropertyComparison {
                        entity_ref: "guard".to_string(),
                        property: "trust".to_string(),
                        operator: ">".to_string(),
                        value: "50".to_string(),
                        annotation: None,
                        span: span("test.urd.md", 41),
                    }),
                ],
                indent_level: 0,
                span: span("test.urd.md", 39),
            }),
            choice("Ask", false),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    let conds = &json["dialogue"]["test/topics"]["conditions"];
    assert!(conds.is_object());
    assert_eq!(conds["any"].as_array().unwrap().len(), 2);
}

// ── Effect Lowering Tests ──

#[test]
fn effect_set_boolean() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Door", make_type_def("Door", vec![], vec![
                make_property_with_default("locked", "boolean", Scalar::Boolean(true)),
            ])),
            fm_entry("door", make_entity_decl("door", "Door", vec![])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            choice_with_content("Unlock", false, vec![set_effect("@door.locked", "false")]),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    let effects = &json["actions"]["test/topics/unlock"]["effects"];
    assert_eq!(effects[0]["set"], "door.locked");
    assert_eq!(effects[0]["to"], false);
}

#[test]
fn effect_set_string() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Guard", make_type_def("Guard", vec![], vec![
                make_property_with_values("mood", "enum", vec!["neutral", "helpful"]),
            ])),
            fm_entry("guard", make_entity_decl("guard", "Guard", vec![])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            choice_with_content("Flatter", false, vec![set_effect("@guard.mood", "helpful")]),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    let effects = &json["actions"]["test/topics/flatter"]["effects"];
    assert_eq!(effects[0]["set"], "guard.mood");
    assert_eq!(effects[0]["to"], "helpful");
}

#[test]
fn effect_arithmetic_add() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Guard", make_type_def("Guard", vec![], vec![
                make_property("trust", "integer"),
            ])),
            fm_entry("guard", make_entity_decl("guard", "Guard", vec![])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            choice_with_content("Compliment", false, vec![
                set_effect_arithmetic("@guard.trust", "+", "10"),
            ]),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    let effects = &json["actions"]["test/topics/compliment"]["effects"];
    assert_eq!(effects[0]["set"], "guard.trust");
    assert_eq!(effects[0]["to"], "guard.trust + 10");
}

#[test]
fn effect_move_to_player() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Key", make_type_def("Key", vec!["portable"], vec![])),
            fm_entry("key", make_entity_decl("key", "Key", vec![])),
        ])),
        vec![
            location("Cell"),
            entity_presence(vec!["key"]),
            section("topics"),
            choice_with_content("Take key", false, vec![move_effect("key", "player")]),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    let effects = &json["actions"]["test/topics/take-key"]["effects"];
    assert_eq!(effects[0]["move"], "key");
    assert_eq!(effects[0]["to"], "player");
}

#[test]
fn effect_move_to_here() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Key", make_type_def("Key", vec!["portable"], vec![])),
            fm_entry("key", make_entity_decl("key", "Key", vec![])),
        ])),
        vec![
            location("Cell"),
            entity_presence(vec!["key"]),
            section("topics"),
            choice_with_content("Drop key", false, vec![move_effect("key", "here")]),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    let effects = &json["actions"]["test/topics/drop-key"]["effects"];
    assert_eq!(effects[0]["move"], "key");
    assert_eq!(effects[0]["to"], "player.container");
}

#[test]
fn effect_move_to_location() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Key", make_type_def("Key", vec!["portable"], vec![])),
            fm_entry("key", make_entity_decl("key", "Key", vec![])),
        ])),
        vec![
            location("Cell"),
            entity_presence(vec!["key"]),
            section("topics"),
            choice_with_content("Send key", false, vec![move_effect("key", "Cell")]),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    let effects = &json["actions"]["test/topics/send-key"]["effects"];
    assert_eq!(effects[0]["move"], "key");
    assert_eq!(effects[0]["to"], "cell");
}

#[test]
fn effect_reveal() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Door", make_type_def("Door", vec![], vec![
                make_property_hidden("prize", "string"),
            ])),
            fm_entry("door", make_entity_decl("door", "Door", vec![])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            choice_with_content("Peek", false, vec![reveal_effect("@door.prize")]),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["actions"]["test/topics/peek"]["effects"][0]["reveal"], "door.prize");
}

#[test]
fn effect_destroy() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Key", make_type_def("Key", vec!["portable"], vec![])),
            fm_entry("key", make_entity_decl("key", "Key", vec![])),
        ])),
        vec![
            location("Cell"),
            entity_presence(vec!["key"]),
            section("topics"),
            choice_with_content("Destroy key", false, vec![destroy_effect("key")]),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["actions"]["test/topics/destroy-key"]["effects"][0]["destroy"], "key");
}

// ── Sequence and Advance Tests ──

#[test]
fn sequence_auto_phase() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![location("Cell"), sequence_heading("Main Quest"), phase_heading("Setup", true)],
    );
    let json = emit_json(single_file_cu(ast));
    let phase = &json["sequences"]["main-quest"]["phases"][0];
    assert_eq!(phase["id"], "setup");
    assert_eq!(phase["auto"], true);
    assert_eq!(phase["advance"], "auto");
}

#[test]
fn advance_on_action() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![location("Cell"), sequence_heading("Quest"), phase_heading("Act", false)],
    );
    let (json_str, _) = link_modify_and_emit(single_file_cu(ast), |st| {
        if let Some(seq) = st.sequences.get_mut("quest") {
            if let Some(phase) = seq.phases.get_mut(0) {
                phase.advance = "on_action".to_string();
            }
        }
    });
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(json["sequences"]["quest"]["phases"][0]["advance"], "on_action");
    assert!(json["sequences"]["quest"]["phases"][0].get("auto").is_none());
}

#[test]
fn advance_on_condition_space_free() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![location("Cell"), sequence_heading("Quest"), phase_heading("Wait", false)],
    );
    let (json_str, _) = link_modify_and_emit(single_file_cu(ast), |st| {
        if let Some(seq) = st.sequences.get_mut("quest") {
            if let Some(phase) = seq.phases.get_mut(0) {
                phase.advance = "on_condition guard.mood == helpful".to_string();
            }
        }
    });
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(json["sequences"]["quest"]["phases"][0]["advance"], "on_condition guard.mood==helpful");
}

#[test]
fn advance_end() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![location("Cell"), sequence_heading("Quest"), phase_heading("Final", false)],
    );
    let (json_str, _) = link_modify_and_emit(single_file_cu(ast), |st| {
        if let Some(seq) = st.sequences.get_mut("quest") {
            if let Some(phase) = seq.phases.get_mut(0) {
                phase.advance = "end".to_string();
            }
        }
    });
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(json["sequences"]["quest"]["phases"][0]["advance"], "end");
}

#[test]
fn advance_on_condition_with_here() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![location("Cell"), sequence_heading("Quest"), phase_heading("Wait", false)],
    );
    let (json_str, _) = link_modify_and_emit(single_file_cu(ast), |st| {
        if let Some(seq) = st.sequences.get_mut("quest") {
            if let Some(phase) = seq.phases.get_mut(0) {
                phase.advance = "on_condition key.container == player.container".to_string();
            }
        }
    });
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(json["sequences"]["quest"]["phases"][0]["advance"], "on_condition key.container==player.container");
}

// ── Dialogue Block Tests ──

#[test]
fn dialogue_section_with_choices() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            choice("Ask about the harbor", false),
            choice("Ask about the crew", false),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    let sec = &json["dialogue"]["test/topics"];
    assert_eq!(sec["id"], "test/topics");
    let choices = sec["choices"].as_array().unwrap();
    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0]["id"], "test/topics/ask-about-the-harbor");
    assert_eq!(choices[0]["label"], "Ask about the harbor");
    assert_eq!(choices[0]["sticky"], false);
}

#[test]
fn dialogue_sticky_choice() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![location("Cell"), section("topics"), choice("Ask again", true)],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["dialogue"]["test/topics"]["choices"][0]["sticky"], true);
}

#[test]
fn dialogue_choice_with_goto() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            choice_with_content("Leave", false, vec![jump("farewell")]),
            section("farewell"),
            choice("Bye", false),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["dialogue"]["test/topics"]["choices"][0]["goto"], "test/farewell");
}

#[test]
fn dialogue_choice_with_goto_end() {
    // A choice with no Jump produces no "goto" field.
    // (In real parsing, `-> end` would be a Jump with target "end",
    //  but LINK doesn't yet special-case "end" — so we test the no-jump path.)
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![location("Cell"), section("topics"), choice("Done", false)],
    );
    let json = emit_json(single_file_cu(ast));
    assert!(json["dialogue"]["test/topics"]["choices"][0].get("goto").is_none());
}

#[test]
fn dialogue_nested_choices() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            choice_with_content("Ask", false, vec![choice("Follow up", false)]),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    let parent = &json["dialogue"]["test/topics"]["choices"][0];
    let nested = parent["choices"].as_array().unwrap();
    assert_eq!(nested.len(), 1);
    assert_eq!(nested[0]["label"], "Follow up");
}

#[test]
fn dialogue_on_exhausted_speech() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("NPC", make_type_def("NPC", vec![], vec![])),
            fm_entry("arina", make_entity_decl("arina", "NPC", vec![])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            choice("Ask", false),
            entity_speech("arina", "Come back later."),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    let exhausted = &json["dialogue"]["test/topics"]["on_exhausted"];
    assert_eq!(exhausted["speaker"], "arina");
    assert_eq!(exhausted["text"], "Come back later.");
}

#[test]
fn dialogue_on_exhausted_prose() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            choice("Ask", false),
            prose("Nothing more to say."),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    let exhausted = &json["dialogue"]["test/topics"]["on_exhausted"];
    assert!(exhausted.get("speaker").is_none());
    assert_eq!(exhausted["text"], "Nothing more to say.");
}

#[test]
fn dialogue_omit_empty() {
    let json = emit_json(minimal_world_cu());
    assert!(json.get("dialogue").is_none());
}

#[test]
fn dialogue_section_description() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![location("Cell"), section("topics"), prose("The tavern is quiet."), choice("Ask", false)],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["dialogue"]["test/topics"]["description"], "The tavern is quiet.");
}

#[test]
fn dialogue_section_prompt() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("NPC", make_type_def("NPC", vec![], vec![])),
            fm_entry("arina", make_entity_decl("arina", "NPC", vec![])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            entity_speech("arina", "What'll it be?"),
            choice("Ask", false),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    let prompt = &json["dialogue"]["test/topics"]["prompt"];
    assert_eq!(prompt["speaker"], "arina");
    assert_eq!(prompt["text"], "What'll it be?");
}

#[test]
fn dialogue_choice_with_conditions_and_effects() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Door", make_type_def("Door", vec![], vec![
                make_property_with_default("locked", "boolean", Scalar::Boolean(true)),
            ])),
            fm_entry("door", make_entity_decl("door", "Door", vec![])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            choice_with_content("Unlock door", false, vec![
                property_comparison("door", "locked", "==", "true"),
                set_effect("@door.locked", "false"),
            ]),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    let ch = &json["dialogue"]["test/topics"]["choices"][0];
    assert_eq!(ch["conditions"][0], "door.locked == true");
    assert_eq!(ch["effects"][0]["set"], "door.locked");
}

#[test]
fn dialogue_choice_with_entity_target() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Door", make_type_def("Door", vec![], vec![])),
            fm_entry("cell_door", make_entity_decl("cell_door", "Door", vec![])),
        ])),
        vec![location("Cell"), section("topics"), choice_with_target("Use key", "cell_door")],
    );
    let json = emit_json(single_file_cu(ast));
    assert!(json["dialogue"]["test/topics"]["choices"][0].get("goto").is_none());
    assert_eq!(json["actions"]["test/topics/use-key"]["target"], "cell_door");
}

#[test]
fn dialogue_choice_with_type_target() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Door", make_type_def("Door", vec![], vec![])),
        ])),
        vec![location("Cell"), section("topics"), choice_with_target_type("Pick a door", "Door")],
    );
    let json = emit_json(single_file_cu(ast));
    assert!(json["dialogue"]["test/topics"]["choices"][0].get("goto").is_none());
    assert_eq!(json["actions"]["test/topics/pick-a-door"]["target_type"], "Door");
}

#[test]
fn dialogue_on_exhausted_with_goto() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            choice("Ask", false),
            prose("Nothing more to discuss."),
            jump("farewell"),
            section("farewell"),
            choice("Bye", false),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    let exhausted = &json["dialogue"]["test/topics"]["on_exhausted"];
    assert_eq!(exhausted["text"], "Nothing more to discuss.");
    assert_eq!(exhausted["goto"], "test/farewell");
}

#[test]
fn dialogue_no_on_exhausted() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![location("Cell"), section("topics"), choice("Ask", false)],
    );
    let json = emit_json(single_file_cu(ast));
    assert!(json["dialogue"]["test/topics"].get("on_exhausted").is_none());
}

#[test]
fn dialogue_multiple_prose_blocks() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            prose("First paragraph."),
            prose("Second paragraph."),
            choice("Ask", false),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["dialogue"]["test/topics"]["description"], "First paragraph.\n\nSecond paragraph.");
}

#[test]
fn dialogue_section_with_prompt_and_description() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("NPC", make_type_def("NPC", vec![], vec![])),
            fm_entry("arina", make_entity_decl("arina", "NPC", vec![])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            prose("The tavern is quiet."),
            entity_speech("arina", "What'll it be?"),
            choice("Ask", false),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    let sec = &json["dialogue"]["test/topics"];
    assert_eq!(sec["description"], "The tavern is quiet.");
    assert_eq!(sec["prompt"]["speaker"], "arina");
    assert_eq!(sec["prompt"]["text"], "What'll it be?");
}

#[test]
fn dialogue_nested_choice_generates_action() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
        ])),
        vec![
            location("Cell"),
            section("topics"),
            choice_with_content("Ask", false, vec![choice("Insist", false)]),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    // Nested choice should generate an action in the actions block.
    assert!(json["actions"]["test/topics/insist"].is_object());
    // Parent choice should also have an action.
    assert!(json["actions"]["test/topics/ask"].is_object());
}

// ── Determinism Tests ──

#[test]
fn determinism_byte_identical() {
    let make_cu = || {
        let ast = make_file_ast(
            "test.urd.md",
            Some(make_frontmatter(vec![
                fm_entry("world", make_world_block(vec![
                    ("name", Scalar::String("test".to_string())),
                    ("start", Scalar::String("Cell".to_string())),
                ])),
                fm_entry("Key", make_type_def("Key", vec!["portable"], vec![
                    make_property("name", "string"),
                ])),
                fm_entry("key", make_entity_decl("key", "Key", vec![
                    ("name", Scalar::String("Rusty Key".to_string())),
                ])),
            ])),
            vec![
                location("Cell"),
                entity_presence(vec!["key"]),
                section("topics"),
                choice("Take key", false),
            ],
        );
        single_file_cu(ast)
    };
    let (json1, _) = link_and_emit(make_cu());
    let (json2, _) = link_and_emit(make_cu());
    assert_eq!(json1, json2);
}

#[test]
fn determinism_cross_file_ordering() {
    let ast_b = make_file_ast(
        "b.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("TypeX", make_type_def("TypeX", vec![], vec![make_property("x", "boolean")])),
        ])),
        vec![],
    );
    let ast_a = make_file_ast(
        "a.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("import", FrontmatterValue::ImportDecl(ImportDecl {
                path: "./b.urd.md".to_string(),
                span: span("a.urd.md", 1),
            })),
            fm_entry("TypeY", make_type_def("TypeY", vec![], vec![make_property("y", "boolean")])),
        ])),
        vec![location("Cell")],
    );
    let json = emit_json(two_file_cu(ast_b, ast_a));
    let keys: Vec<&String> = json["types"].as_object().unwrap().keys().collect();
    assert_eq!(keys[0], "TypeX");
    assert_eq!(keys[1], "TypeY");
}

#[test]
fn determinism_key_order_entity() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Key", make_type_def("Key", vec!["portable"], vec![make_property("name", "string")])),
            fm_entry("key", make_entity_decl("key", "Key", vec![
                ("name", Scalar::String("Rusty Key".to_string())),
            ])),
        ])),
        vec![location("Cell")],
    );
    let json = emit_json(single_file_cu(ast));
    // Entity keys should be in order: "type", then "properties".
    let entity = json["entities"]["key"].as_object().unwrap();
    let keys: Vec<&String> = entity.keys().collect();
    assert_eq!(keys[0], "type");
    assert_eq!(keys[1], "properties");
}

#[test]
fn determinism_top_level_key_order() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("test".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Key", make_type_def("Key", vec!["portable"], vec![make_property("name", "string")])),
            fm_entry("key", make_entity_decl("key", "Key", vec![])),
        ])),
        vec![
            location("Cell"),
            entity_presence(vec!["key"]),
            section("topics"),
            choice("Ask", false),
        ],
    );
    let (json_str, _) = link_and_emit(single_file_cu(ast));
    let world_pos = json_str.find("\"world\"").unwrap();
    let types_pos = json_str.find("\"types\"").unwrap();
    let entities_pos = json_str.find("\"entities\"").unwrap();
    let locations_pos = json_str.find("\"locations\"").unwrap();
    assert!(world_pos < types_pos);
    assert!(types_pos < entities_pos);
    assert!(entities_pos < locations_pos);
}

// ── Integration Tests ──

#[test]
fn integration_two_room_key_puzzle() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("key-puzzle".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("Key", make_type_def("Key", vec!["portable"], vec![
                make_property("name", "string"),
            ])),
            fm_entry("Door", make_type_def("Door", vec![], vec![
                make_property_with_default("locked", "boolean", Scalar::Boolean(true)),
            ])),
            fm_entry("rusty_key", make_entity_decl("rusty_key", "Key", vec![
                ("name", Scalar::String("Rusty Key".to_string())),
            ])),
            fm_entry("cell_door", make_entity_decl("cell_door", "Door", vec![])),
        ])),
        vec![
            location("Cell"),
            prose("A dim stone cell."),
            entity_presence(vec!["rusty_key", "cell_door"]),
            exit_decl_with_children("north", "Corridor", vec![
                property_comparison("cell_door", "locked", "==", "false"),
                blocked_message("The door is locked."),
            ]),
            section("topics"),
            choice_with_content("Take key", false, vec![move_effect("rusty_key", "player")]),
            choice_with_content("Unlock door", false, vec![
                containment_check("rusty_key", "player", false),
                set_effect("@cell_door.locked", "false"),
            ]),
            location("Corridor"),
            prose("A long corridor stretches north."),
        ],
    );
    let json = emit_json(single_file_cu(ast));
    assert_eq!(json["world"]["name"], "key-puzzle");
    assert_eq!(json["world"]["urd"], "1");
    assert_eq!(json["world"]["start"], "cell");
    assert!(json["types"]["Key"].is_object());
    assert!(json["types"]["Door"].is_object());
    assert_eq!(json["entities"]["rusty_key"]["type"], "Key");
    assert_eq!(json["entities"]["rusty_key"]["properties"]["name"], "Rusty Key");
    assert_eq!(json["locations"]["cell"]["description"], "A dim stone cell.");
    assert_eq!(json["locations"]["cell"]["contains"][0], "rusty_key");
    assert_eq!(json["locations"]["cell"]["exits"]["north"]["to"], "corridor");
    assert_eq!(json["locations"]["cell"]["exits"]["north"]["condition"], "cell_door.locked == false");
    assert_eq!(json["locations"]["cell"]["exits"]["north"]["blocked_message"], "The door is locked.");
    assert!(json["actions"]["test/topics/take-key"].is_object());
    assert!(json["actions"]["test/topics/unlock-door"].is_object());
    assert!(json["dialogue"]["test/topics"]["choices"].is_array());
}

#[test]
fn integration_multi_file() {
    // Build imported file with correct file paths in spans.
    let items_span = |line: u32| Span::new("items.urd.md".to_string(), line, 1, line, 80);
    let key_type = FrontmatterValue::TypeDef(TypeDef {
        name: "Key".to_string(),
        traits: vec!["portable".to_string()],
        properties: vec![PropertyDef {
            name: "name".to_string(),
            property_type: "string".to_string(),
            default: None,
            visibility: None,
            values: None,
            min: None,
            max: None,
            ref_type: None,
            element_type: None,
            element_values: None,
            element_ref_type: None,
            description: None,
            span: items_span(3),
        }],
        span: items_span(2),
    });
    let key_entity = FrontmatterValue::EntityDecl(EntityDecl {
        id: "rusty_key".to_string(),
        type_name: "Key".to_string(),
        property_overrides: vec![],
        annotation: None,
        span: items_span(4),
    });
    let ast_b = make_file_ast(
        "items.urd.md",
        Some(Frontmatter {
            entries: vec![
                FrontmatterEntry { key: "Key".to_string(), value: key_type, span: items_span(2) },
                FrontmatterEntry { key: "rusty_key".to_string(), value: key_entity, span: items_span(4) },
            ],
            span: items_span(1),
        }),
        vec![],
    );
    let ast_a = make_file_ast(
        "main.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("name", Scalar::String("multi".to_string())),
                ("start", Scalar::String("Cell".to_string())),
            ])),
            fm_entry("import", FrontmatterValue::ImportDecl(ImportDecl {
                path: "./items.urd.md".to_string(),
                span: span("main.urd.md", 1),
            })),
        ])),
        vec![location("Cell"), entity_presence(vec!["rusty_key"])],
    );
    let json = emit_json(two_file_cu(ast_b, ast_a));
    assert_eq!(json["world"]["name"], "multi");
    assert!(json["types"]["Key"].is_object());
    assert!(json["entities"]["rusty_key"].is_object());
    assert_eq!(json["locations"]["cell"]["contains"][0], "rusty_key");
}

#[test]
fn integration_empty_world() {
    let json = emit_json(minimal_world_cu());
    assert_eq!(json["world"]["name"], "test");
    assert!(json.get("types").is_none());
    assert!(json.get("entities").is_none());
    assert!(json.get("rules").is_none());
    assert!(json.get("actions").is_none());
    assert!(json.get("sequences").is_none());
    assert!(json.get("dialogue").is_none());
    assert!(json.get("locations").is_some());
}

#[test]
fn integration_trailing_newline() {
    let (json_str, _) = link_and_emit(minimal_world_cu());
    assert!(json_str.ends_with("}\n"));
    assert!(!json_str.ends_with("}}\n"));
}
