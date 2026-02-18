// Tests for Phase 3: LINK
//
// Test categories from the LINK phase brief:
// 1. Collection (10)
// 2. Choice-to-action (7)
// 3. Resolution (14)
// 4. ID derivation (6)
// 5. Integration (3)
// 6. Error recovery (4)
// 7. Scope & location context (6)

use indexmap::IndexMap;
use urd_compiler::ast::*;
use urd_compiler::diagnostics::DiagnosticCollector;
use urd_compiler::graph::{CompilationUnit, DependencyGraph, FileNode};
use urd_compiler::link;
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

fn make_type_def_in(name: &str, file: &str, line: u32, traits: Vec<&str>, properties: Vec<PropertyDef>) -> FrontmatterValue {
    FrontmatterValue::TypeDef(TypeDef {
        name: name.to_string(),
        traits: traits.into_iter().map(String::from).collect(),
        properties,
        span: span(file, line),
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
        name: name.to_string(),
        property_type: prop_type.to_string(),
        default: Some(default),
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

fn make_entity_decl(id: &str, type_name: &str, overrides: Vec<(&str, Scalar)>) -> FrontmatterValue {
    FrontmatterValue::EntityDecl(EntityDecl {
        id: id.to_string(),
        type_name: type_name.to_string(),
        property_overrides: overrides.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
        annotation: None,
        span: span("test.urd.md", 4),
    })
}

fn make_entity_decl_in(id: &str, type_name: &str, file: &str, line: u32, overrides: Vec<(&str, Scalar)>) -> FrontmatterValue {
    FrontmatterValue::EntityDecl(EntityDecl {
        id: id.to_string(),
        type_name: type_name.to_string(),
        property_overrides: overrides.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
        annotation: None,
        span: span(file, line),
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

fn location_in(name: &str, file: &str, line: u32) -> ContentNode {
    ContentNode::LocationHeading(LocationHeading {
        display_name: name.to_string(),
        span: span(file, line),
    })
}

fn section(name: &str) -> ContentNode {
    ContentNode::SectionLabel(SectionLabel {
        name: name.to_string(),
        span: span("test.urd.md", 15),
    })
}

fn section_in(name: &str, file: &str, line: u32) -> ContentNode {
    ContentNode::SectionLabel(SectionLabel {
        name: name.to_string(),
        span: span(file, line),
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

fn choice_with_type_target(label: &str, target_type: &str) -> ContentNode {
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

fn choice_with_children(label: &str, children: Vec<ContentNode>) -> ContentNode {
    ContentNode::Choice(Choice {
        sticky: false,
        label: label.to_string(),
        target: None,
        target_type: None,
        content: children,
        indent_level: 1,
        annotation: None,
        span: span("test.urd.md", 20),
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

fn entity_presence(refs: Vec<&str>) -> ContentNode {
    let len = refs.len();
    ContentNode::EntityPresence(EntityPresence {
        entity_refs: refs.into_iter().map(String::from).collect(),
        annotations: vec![None; len],
        span: span("test.urd.md", 12),
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

fn stage_direction(entity_ref: &str, text: &str) -> ContentNode {
    ContentNode::StageDirection(StageDirection {
        entity_ref: entity_ref.to_string(),
        text: text.to_string(),
        annotation: None,
        span: span("test.urd.md", 31),
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

fn exit_jump(target: &str) -> ContentNode {
    ContentNode::Jump(Jump {
        target: target.to_string(),
        is_exit_qualified: true,
        indent_level: 0,
        annotation: None,
        span: span("test.urd.md", 36),
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

fn destroy_effect(entity_ref: &str) -> ContentNode {
    ContentNode::Effect(Effect {
        effect_type: EffectType::Destroy {
            entity_ref: entity_ref.to_string(),
        },
        indent_level: 0,
        annotation: None,
        span: span("test.urd.md", 47),
    })
}

fn error_node(text: &str) -> ContentNode {
    ContentNode::ErrorNode(ErrorNode {
        raw_text: text.to_string(),
        attempted_rule: None,
        span: span("test.urd.md", 50),
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

/// Build a single-file CompilationUnit.
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

/// Build a two-file CompilationUnit (file_b imported by file_a).
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

/// Build a two-file CompilationUnit where file_a does NOT import file_b.
fn two_file_cu_no_import(ast_b: FileAst, ast_a: FileAst) -> CompilationUnit {
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
            imports: Vec::new(),
        },
    );
    CompilationUnit {
        graph: DependencyGraph {
            nodes,
            edges: Vec::new(),
            entry_path: Some(path_a.clone()),
        },
        ordered_asts: vec![path_b, path_a],
    }
}

fn has_error(diag: &DiagnosticCollector, code: &str) -> bool {
    diag.all().iter().any(|d| d.code == code)
}

fn error_count(diag: &DiagnosticCollector, code: &str) -> usize {
    diag.all().iter().filter(|d| d.code == code).count()
}

fn has_warning(diag: &DiagnosticCollector, code: &str) -> bool {
    diag.all().iter().any(|d| d.code == code && d.severity == urd_compiler::diagnostics::Severity::Warning)
}

fn has_suggestion(diag: &DiagnosticCollector, code: &str) -> bool {
    diag.all().iter().any(|d| d.code == code && d.suggestion.is_some())
}

// ═══════════════════════════════════════════════════════════════════
// 1. Collection Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn collect_single_type() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Guard", make_type_def("Guard", vec!["character"], vec![
                make_property("mood", "string"),
                make_property_with_default("health", "integer", Scalar::Integer(100)),
            ])),
        ])),
        Vec::new(),
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    assert!(linked.symbol_table.types.contains_key("Guard"));
    let ts = &linked.symbol_table.types["Guard"];
    assert_eq!(ts.traits, vec!["character"]);
    assert_eq!(ts.properties.len(), 2);
    assert!(ts.properties.contains_key("mood"));
    assert!(ts.properties.contains_key("health"));
}

#[test]
fn collect_single_entity() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Guard", make_type_def("Guard", vec![], vec![
                make_property("mood", "string"),
            ])),
            fm_entry("guard", make_entity_decl("guard", "Guard", vec![
                ("mood", Scalar::String("grumpy".to_string())),
            ])),
        ])),
        Vec::new(),
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    assert!(linked.symbol_table.entities.contains_key("guard"));
    let es = &linked.symbol_table.entities["guard"];
    assert_eq!(es.type_name, "Guard");
    assert_eq!(es.property_overrides.len(), 1);
}

#[test]
fn collect_location_from_heading() {
    let ast = make_file_ast(
        "test.urd.md",
        None,
        vec![location("The Tavern")],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    assert!(linked.symbol_table.locations.contains_key("the-tavern"));
    let ls = &linked.symbol_table.locations["the-tavern"];
    assert_eq!(ls.display_name, "The Tavern");
    assert_eq!(ls.id, "the-tavern");
}

#[test]
fn collect_section_with_choices() {
    let ast = make_file_ast(
        "tavern.urd.md",
        None,
        vec![
            section("topics"),
            choice("Ask about the harbor", false),
            choice("Order a drink", true),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    assert!(linked.symbol_table.sections.contains_key("tavern/topics"));
    let sec = &linked.symbol_table.sections["tavern/topics"];
    assert_eq!(sec.local_name, "topics");
    assert_eq!(sec.compiled_id, "tavern/topics");
    assert_eq!(sec.choices.len(), 2);
    assert_eq!(sec.choices[0].compiled_id, "tavern/topics/ask-about-the-harbor");
    assert_eq!(sec.choices[1].compiled_id, "tavern/topics/order-a-drink");
    assert!(!sec.choices[0].sticky);
    assert!(sec.choices[1].sticky);
}

#[test]
fn collect_duplicate_entity() {
    let ast_a = make_file_ast(
        "a.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("guard", make_entity_decl_in("guard", "Guard", "a.urd.md", 5, vec![])),
        ])),
        Vec::new(),
    );
    let ast_b = make_file_ast(
        "b.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Guard", make_type_def_in("Guard", "b.urd.md", 2, vec![], vec![])),
            fm_entry("guard", make_entity_decl_in("guard", "Guard", "b.urd.md", 3, vec![])),
        ])),
        Vec::new(),
    );
    let cu = two_file_cu(ast_b, ast_a);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD302"));
    // First declaration wins.
    assert!(linked.symbol_table.entities.contains_key("guard"));
    assert_eq!(linked.symbol_table.duplicates.len(), 1);
    assert_eq!(linked.symbol_table.duplicates[0].namespace, "entities");
}

#[test]
fn collect_duplicate_type() {
    let ast_a = make_file_ast(
        "a.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Guard", make_type_def_in("Guard", "a.urd.md", 2, vec![], vec![])),
        ])),
        Vec::new(),
    );
    let ast_b = make_file_ast(
        "b.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Guard", make_type_def_in("Guard", "b.urd.md", 2, vec![], vec![])),
        ])),
        Vec::new(),
    );
    let cu = two_file_cu(ast_b, ast_a);
    let mut diag = DiagnosticCollector::new();
    let _linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD303"));
}

#[test]
fn collect_duplicate_location() {
    let ast = make_file_ast(
        "test.urd.md",
        None,
        vec![
            location("Cell"),
            location("cell"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let _linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD304"));
}

#[test]
fn collect_duplicate_section_in_same_file() {
    let ast = make_file_ast(
        "tavern.urd.md",
        None,
        vec![
            section("topics"),
            section("topics"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let _linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD305"));
}

#[test]
fn collect_duplicate_choice_slugs() {
    let ast = make_file_ast(
        "tavern.urd.md",
        None,
        vec![
            section("topics"),
            choice("Ask why", false),
            choice("Ask Why", false),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let _linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD306"));
}

#[test]
fn collect_error_node_skipped() {
    let ast = make_file_ast(
        "tavern.urd.md",
        None,
        vec![
            section("greetings"),
            error_node("broken stuff"),
            section("farewell"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    assert!(linked.symbol_table.sections.contains_key("tavern/greetings"));
    assert!(linked.symbol_table.sections.contains_key("tavern/farewell"));
}

// ═══════════════════════════════════════════════════════════════════
// 2. Choice-to-Action Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn choice_generates_action() {
    let ast = make_file_ast(
        "tavern.urd.md",
        None,
        vec![
            section("topics"),
            choice("Ask about the harbor", false),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    assert!(linked.symbol_table.actions.contains_key("tavern/topics/ask-about-the-harbor"));
}

#[test]
fn choice_with_entity_target() {
    let ast = make_file_ast(
        "tavern.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Container", make_type_def("Container", vec![], vec![])),
            fm_entry("cell_door", make_entity_decl("cell_door", "Container", vec![])),
        ])),
        vec![
            section("actions"),
            choice_with_target("Use key", "cell_door"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    let action = &linked.symbol_table.actions["tavern/actions/use-key"];
    assert_eq!(action.target, Some("cell_door".to_string()));
    assert_eq!(action.target_type, None);
}

#[test]
fn choice_with_type_target_action() {
    let ast = make_file_ast(
        "tavern.urd.md",
        None,
        vec![
            section("actions"),
            choice_with_type_target("Pick a door", "Door"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    let action = &linked.symbol_table.actions["tavern/actions/pick-a-door"];
    assert_eq!(action.target, None);
    assert_eq!(action.target_type, Some("Door".to_string()));
}

#[test]
fn choice_with_no_target() {
    let ast = make_file_ast(
        "tavern.urd.md",
        None,
        vec![
            section("topics"),
            choice("Ask about the weather", false),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    let action = &linked.symbol_table.actions["tavern/topics/ask-about-the-weather"];
    assert_eq!(action.target, None);
    assert_eq!(action.target_type, None);
}

#[test]
fn nested_choice_generates_action() {
    let inner = choice("Follow up", false);
    let ast = make_file_ast(
        "tavern.urd.md",
        None,
        vec![
            section("topics"),
            choice_with_children("Ask about the harbor", vec![inner]),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(linked.symbol_table.actions.contains_key("tavern/topics/ask-about-the-harbor"));
    assert!(linked.symbol_table.actions.contains_key("tavern/topics/follow-up"));
}

#[test]
fn choice_action_id_matches_choice_id() {
    let ast = make_file_ast(
        "tavern.urd.md",
        None,
        vec![
            section("topics"),
            choice("Ask about the harbor", false),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    let sec = &linked.symbol_table.sections["tavern/topics"];
    let choice_id = &sec.choices[0].compiled_id;
    assert!(linked.symbol_table.actions.contains_key(choice_id));
    assert_eq!(choice_id, "tavern/topics/ask-about-the-harbor");
}

#[test]
fn topological_order_respected() {
    // File B declares type Guard, file A declares entity @guard: Guard.
    // B should be processed first.
    let ast_b = make_file_ast(
        "types.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Guard", make_type_def_in("Guard", "types.urd.md", 2, vec![], vec![
                make_property("mood", "string"),
            ])),
        ])),
        Vec::new(),
    );
    let ast_a = make_file_ast(
        "main.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("guard", make_entity_decl_in("guard", "Guard", "main.urd.md", 3, vec![])),
        ])),
        Vec::new(),
    );
    let cu = two_file_cu(ast_b, ast_a);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    // Guard type should be registered before @guard entity.
    assert!(linked.symbol_table.types.contains_key("Guard"));
    assert!(linked.symbol_table.entities.contains_key("guard"));
    // Type should be resolved on the entity.
    assert_eq!(linked.symbol_table.entities["guard"].type_symbol, Some("Guard".to_string()));
}

// ═══════════════════════════════════════════════════════════════════
// 3. Resolution Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn resolve_entity_reference() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Guard", make_type_def("Guard", vec![], vec![])),
            fm_entry("guard", make_entity_decl("guard", "Guard", vec![])),
        ])),
        vec![entity_speech("guard", "Halt!")],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    if let ContentNode::EntitySpeech(speech) = &linked.graph.nodes["test.urd.md"].ast.content[0] {
        assert!(speech.annotation.is_some());
        assert_eq!(speech.annotation.as_ref().unwrap().resolved_entity, Some("guard".to_string()));
    } else {
        panic!("expected EntitySpeech");
    }
}

#[test]
fn resolve_type_on_entity() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Guard", make_type_def("Guard", vec![], vec![
                make_property("mood", "string"),
            ])),
            fm_entry("guard", make_entity_decl("guard", "Guard", vec![])),
        ])),
        Vec::new(),
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    assert_eq!(linked.symbol_table.entities["guard"].type_symbol, Some("Guard".to_string()));
}

#[test]
fn resolve_property_access() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Guard", make_type_def("Guard", vec![], vec![
                make_property("mood", "string"),
            ])),
            fm_entry("guard", make_entity_decl("guard", "Guard", vec![])),
        ])),
        vec![
            property_comparison("guard", "mood", "==", "happy"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    if let ContentNode::Condition(cond) = &linked.graph.nodes["test.urd.md"].ast.content[0] {
        if let ConditionExpr::PropertyComparison(pc) = &cond.expr {
            assert!(pc.annotation.is_some());
            let ann = pc.annotation.as_ref().unwrap();
            assert_eq!(ann.resolved_entity, Some("guard".to_string()));
            assert_eq!(ann.resolved_property, Some("mood".to_string()));
            assert_eq!(ann.resolved_type, Some("Guard".to_string()));
        } else {
            panic!("expected PropertyComparison");
        }
    } else {
        panic!("expected Condition");
    }
}

#[test]
fn resolve_unresolved_entity() {
    let ast = make_file_ast(
        "test.urd.md",
        None,
        vec![entity_speech("missing", "Hello")],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let _linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD301"));
}

#[test]
fn resolve_unresolved_entity_with_suggestion() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Guard", make_type_def("Guard", vec![], vec![])),
            fm_entry("guard", make_entity_decl("guard", "Guard", vec![])),
        ])),
        vec![entity_speech("gaurd", "Halt!")],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let _linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD301"));
    assert!(has_suggestion(&diag, "URD301"));
}

#[test]
fn resolve_scope_enforcement() {
    // guard declared in B, file A does NOT import B.
    let ast_b = make_file_ast(
        "npcs.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Guard", make_type_def_in("Guard", "npcs.urd.md", 2, vec![], vec![])),
            fm_entry("guard", make_entity_decl_in("guard", "Guard", "npcs.urd.md", 3, vec![])),
        ])),
        Vec::new(),
    );
    let ast_a = make_file_ast(
        "main.urd.md",
        None,
        vec![entity_speech("guard", "Hello")],
    );
    let cu = two_file_cu_no_import(ast_b, ast_a);
    let mut diag = DiagnosticCollector::new();
    let _linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD301"));
    // Should hint about missing import.
    let err = diag.all().iter().find(|d| d.code == "URD301").unwrap();
    assert!(err.suggestion.as_ref().unwrap().contains("not imported"));
}

#[test]
fn resolve_jump_to_section() {
    let ast = make_file_ast(
        "tavern.urd.md",
        None,
        vec![
            section("topics"),
            jump("topics"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    if let ContentNode::Jump(j) = &linked.graph.nodes["tavern.urd.md"].ast.content[1] {
        assert!(j.annotation.is_some());
        assert_eq!(
            j.annotation.as_ref().unwrap().resolved_section,
            Some("tavern/topics".to_string()),
        );
    } else {
        panic!("expected Jump");
    }
}

#[test]
fn resolve_jump_to_exit() {
    let ast = make_file_ast(
        "test.urd.md",
        None,
        vec![
            location("Cell"),
            exit_decl("north", "Harbor"),
            jump("north"),
            location("Harbor"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    if let ContentNode::Jump(j) = &linked.graph.nodes["test.urd.md"].ast.content[2] {
        assert!(j.annotation.is_some());
        // Resolves to exit → location context "cell".
        assert_eq!(
            j.annotation.as_ref().unwrap().resolved_location,
            Some("cell".to_string()),
        );
    } else {
        panic!("expected Jump");
    }
}

#[test]
fn resolve_jump_shadowing() {
    // Both a section named "north" and an exit named "north".
    let ast = make_file_ast(
        "test.urd.md",
        None,
        vec![
            location("Cell"),
            exit_decl("north", "Harbor"),
            section("north"),
            jump("north"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    // Should resolve to section with URD310 warning.
    assert!(has_warning(&diag, "URD310"));
    if let ContentNode::Jump(j) = &linked.graph.nodes["test.urd.md"].ast.content[3] {
        assert!(j.annotation.is_some());
        assert_eq!(
            j.annotation.as_ref().unwrap().resolved_section,
            Some("test/north".to_string()),
        );
    } else {
        panic!("expected Jump");
    }
}

#[test]
fn resolve_explicit_exit_jump() {
    let ast = make_file_ast(
        "test.urd.md",
        None,
        vec![
            location("Cell"),
            exit_decl("north", "Harbor"),
            exit_jump("north"),
            location("Harbor"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    if let ContentNode::Jump(j) = &linked.graph.nodes["test.urd.md"].ast.content[2] {
        assert!(j.annotation.is_some());
    } else {
        panic!("expected Jump");
    }
}

#[test]
fn resolve_unresolved_jump() {
    let ast = make_file_ast(
        "test.urd.md",
        None,
        vec![jump("nowhere")],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let _linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD309"));
}

#[test]
fn resolve_exit_destination() {
    let ast = make_file_ast(
        "test.urd.md",
        None,
        vec![
            location("Cell"),
            exit_decl("north", "Harbor"),
            location("Harbor"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    let cell = &linked.symbol_table.locations["cell"];
    assert!(cell.exits.contains_key("north"));
    assert_eq!(cell.exits["north"].resolved_destination, Some("harbor".to_string()));
}

#[test]
fn resolve_unresolved_exit_destination() {
    let ast = make_file_ast(
        "test.urd.md",
        None,
        vec![
            location("Cell"),
            exit_decl("north", "Nowhere Special"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let _linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD312"));
}

#[test]
fn resolve_entity_presence() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Key", make_type_def("Key", vec![], vec![])),
            fm_entry("rusty_key", make_entity_decl("rusty_key", "Key", vec![])),
            fm_entry("Door", make_type_def("Door", vec![], vec![])),
            fm_entry("cell_door", make_entity_decl("cell_door", "Door", vec![])),
        ])),
        vec![
            location("Cell"),
            entity_presence(vec!["rusty_key", "cell_door"]),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    let cell = &linked.symbol_table.locations["cell"];
    assert_eq!(cell.contains.len(), 2);
    assert!(cell.contains.contains(&"rusty_key".to_string()));
    assert!(cell.contains.contains(&"cell_door".to_string()));
}

// ═══════════════════════════════════════════════════════════════════
// 4. ID Derivation Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn id_section() {
    let ast = make_file_ast(
        "tavern.urd.md",
        None,
        vec![section("topics")],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(linked.symbol_table.sections.contains_key("tavern/topics"));
}

#[test]
fn id_choice() {
    let ast = make_file_ast(
        "tavern.urd.md",
        None,
        vec![
            section("topics"),
            choice("Ask about the harbor", false),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    let sec = &linked.symbol_table.sections["tavern/topics"];
    assert_eq!(sec.choices[0].compiled_id, "tavern/topics/ask-about-the-harbor");
}

#[test]
fn id_location() {
    let ast = make_file_ast(
        "test.urd.md",
        None,
        vec![location("The Rusty Anchor")],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(linked.symbol_table.locations.contains_key("the-rusty-anchor"));
}

#[test]
fn id_slugify_special_chars() {
    let ast = make_file_ast(
        "test.urd.md",
        None,
        vec![location("Café & Bar!!!")],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(linked.symbol_table.locations.contains_key("caf-bar"));
}

#[test]
fn id_slugify_collapse() {
    let ast = make_file_ast(
        "test.urd.md",
        None,
        vec![location("-- hello -- world --")],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(linked.symbol_table.locations.contains_key("hello-world"));
}

#[test]
fn id_empty_slug() {
    let ast = make_file_ast(
        "test.urd.md",
        None,
        vec![location("!!!")],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let _linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD313"));
}

// ═══════════════════════════════════════════════════════════════════
// 5. Integration Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn integration_two_room_key_puzzle() {
    // Single file with: types, entities, two locations, exits, entity presence, conditions.
    let file = "dungeon.urd.md";
    let ast = make_file_ast(
        file,
        Some(Frontmatter {
            entries: vec![
                fm_entry("Container", make_type_def_in("Container", file, 2, vec![], vec![
                    make_property_with_default("locked", "boolean", Scalar::Boolean(false)),
                ])),
                fm_entry("Item", make_type_def_in("Item", file, 3, vec!["portable"], vec![])),
                fm_entry("cell_door", make_entity_decl_in("cell_door", "Container", file, 4, vec![
                    ("locked", Scalar::Boolean(true)),
                ])),
                fm_entry("rusty_key", make_entity_decl_in("rusty_key", "Item", file, 5, vec![])),
            ],
            span: span(file, 1),
        }),
        vec![
            location_in("Cell", file, 10),
            entity_presence(vec!["rusty_key", "cell_door"]),
            exit_decl("north", "Corridor"),
            section_in("examine", file, 14),
            entity_speech("cell_door", "A heavy iron door."),
            property_comparison("cell_door", "locked", "==", "true"),
            location_in("Corridor", file, 20),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    assert_eq!(linked.symbol_table.types.len(), 2);
    assert_eq!(linked.symbol_table.entities.len(), 2);
    assert_eq!(linked.symbol_table.locations.len(), 2);
    assert_eq!(linked.symbol_table.sections.len(), 1);

    let cell = &linked.symbol_table.locations["cell"];
    assert_eq!(cell.contains.len(), 2);
    assert!(cell.exits.contains_key("north"));
    assert_eq!(cell.exits["north"].resolved_destination, Some("corridor".to_string()));
}

#[test]
fn integration_multi_file_project() {
    // File B: types and entities. File A: imports B, references entities.
    let ast_b = make_file_ast(
        "types.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Guard", make_type_def_in("Guard", "types.urd.md", 2, vec![], vec![
                make_property("mood", "string"),
            ])),
            fm_entry("guard", make_entity_decl_in("guard", "Guard", "types.urd.md", 3, vec![])),
        ])),
        Vec::new(),
    );
    let ast_a = make_file_ast(
        "main.urd.md",
        None,
        vec![
            entity_speech("guard", "Halt!"),
            stage_direction("guard", "draws sword"),
        ],
    );
    let cu = two_file_cu(ast_b, ast_a);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    // Entity should be resolved in main.urd.md content.
    if let ContentNode::EntitySpeech(speech) = &linked.graph.nodes["main.urd.md"].ast.content[0] {
        assert_eq!(speech.annotation.as_ref().unwrap().resolved_entity, Some("guard".to_string()));
    }
}

#[test]
fn integration_jump_disambiguation() {
    // File has section and exit sharing a name. Section should win with warning.
    let ast = make_file_ast(
        "test.urd.md",
        None,
        vec![
            location("Room"),
            exit_decl("north", "Room"),
            section("north"),
            jump("north"),
            exit_jump("north"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(has_warning(&diag, "URD310"));
    // Standard jump resolves to section.
    if let ContentNode::Jump(j) = &linked.graph.nodes["test.urd.md"].ast.content[3] {
        assert_eq!(j.annotation.as_ref().unwrap().resolved_section, Some("test/north".to_string()));
    }
    // Explicit exit jump resolves to exit.
    if let ContentNode::Jump(j) = &linked.graph.nodes["test.urd.md"].ast.content[4] {
        assert!(j.annotation.is_some());
    }
}

// ═══════════════════════════════════════════════════════════════════
// 6. Error Recovery Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn recovery_one_bad_entity_among_good() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Guard", make_type_def("Guard", vec![], vec![
                make_property("mood", "string"),
            ])),
            fm_entry("guard", make_entity_decl("guard", "Guard", vec![])),
            fm_entry("innkeeper", make_entity_decl("innkeeper", "MissingType", vec![])),
        ])),
        Vec::new(),
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD307"));
    assert_eq!(error_count(&diag, "URD307"), 1);
    // Guard entity should still be fully resolved.
    assert_eq!(linked.symbol_table.entities["guard"].type_symbol, Some("Guard".to_string()));
    // Innkeeper entity's type should be unresolved.
    assert_eq!(linked.symbol_table.entities["innkeeper"].type_symbol, None);
}

#[test]
fn recovery_cascading_suppressed() {
    // Entity references a missing type. Three property accesses on that entity.
    // Should get URD307 once, zero URD308.
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("guard", make_entity_decl("guard", "MissingType", vec![])),
        ])),
        vec![
            property_comparison("guard", "mood", "==", "happy"),
            property_comparison("guard", "health", ">=", "50"),
            property_comparison("guard", "strength", "<", "10"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let _linked = link::link(cu, &mut diag);

    assert_eq!(error_count(&diag, "URD307"), 1);
    assert_eq!(error_count(&diag, "URD308"), 0);
}

#[test]
fn recovery_error_node_among_valid() {
    let ast = make_file_ast(
        "tavern.urd.md",
        None,
        vec![
            section("greetings"),
            error_node("broken stuff"),
            section("farewell"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    assert!(linked.symbol_table.sections.contains_key("tavern/greetings"));
    assert!(linked.symbol_table.sections.contains_key("tavern/farewell"));
}

#[test]
fn recovery_duplicate_then_reference() {
    // Two files declare @guard, then it's referenced. First should win, reference resolves.
    let ast_b = make_file_ast(
        "b.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Guard", make_type_def_in("Guard", "b.urd.md", 2, vec![], vec![])),
            fm_entry("guard", make_entity_decl_in("guard", "Guard", "b.urd.md", 3, vec![])),
        ])),
        Vec::new(),
    );
    let ast_a = make_file_ast(
        "a.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("guard", make_entity_decl_in("guard", "Guard", "a.urd.md", 2, vec![])),
        ])),
        vec![entity_speech("guard", "Halt!")],
    );
    let cu = two_file_cu(ast_b, ast_a);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    // URD302 for duplicate.
    assert!(has_error(&diag, "URD302"));
    // Reference should still resolve to first declaration (from b.urd.md).
    if let ContentNode::EntitySpeech(speech) = &linked.graph.nodes["a.urd.md"].ast.content[0] {
        assert!(speech.annotation.is_some());
        assert_eq!(speech.annotation.as_ref().unwrap().resolved_entity, Some("guard".to_string()));
    }
    // No URD301.
    assert!(!has_error(&diag, "URD301"));
}

// ═══════════════════════════════════════════════════════════════════
// 7. Scope & Location Context Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn scope_exit_outside_location_context() {
    let ast = make_file_ast(
        "test.urd.md",
        None,
        vec![
            exit_decl("north", "Harbor"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let _linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD314"));
}

#[test]
fn scope_entity_presence_outside_location() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Key", make_type_def("Key", vec![], vec![])),
            fm_entry("rusty_key", make_entity_decl("rusty_key", "Key", vec![])),
        ])),
        vec![
            entity_presence(vec!["rusty_key"]),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let _linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD314"));
}

#[test]
fn scope_explicit_exit_jump_outside_location() {
    let ast = make_file_ast(
        "test.urd.md",
        None,
        vec![exit_jump("north")],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let _linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD314"));
}

#[test]
fn scope_cross_file_with_import() {
    let ast_b = make_file_ast(
        "types.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Guard", make_type_def_in("Guard", "types.urd.md", 2, vec![], vec![])),
            fm_entry("guard", make_entity_decl_in("guard", "Guard", "types.urd.md", 3, vec![])),
        ])),
        Vec::new(),
    );
    let ast_a = make_file_ast(
        "main.urd.md",
        None,
        vec![entity_speech("guard", "Hello")],
    );
    let cu = two_file_cu(ast_b, ast_a);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    if let ContentNode::EntitySpeech(speech) = &linked.graph.nodes["main.urd.md"].ast.content[0] {
        assert_eq!(speech.annotation.as_ref().unwrap().resolved_entity, Some("guard".to_string()));
    }
}

#[test]
fn scope_cross_file_without_import() {
    let ast_b = make_file_ast(
        "npcs.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Guard", make_type_def_in("Guard", "npcs.urd.md", 2, vec![], vec![])),
            fm_entry("guard", make_entity_decl_in("guard", "Guard", "npcs.urd.md", 3, vec![])),
        ])),
        Vec::new(),
    );
    let ast_a = make_file_ast(
        "main.urd.md",
        None,
        vec![entity_speech("guard", "Hello")],
    );
    let cu = two_file_cu_no_import(ast_b, ast_a);
    let mut diag = DiagnosticCollector::new();
    let _linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD301"));
}

#[test]
fn scope_forward_reference_within_file() {
    // Entity referenced before declaration in same file. Should resolve.
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Guard", make_type_def("Guard", vec![], vec![])),
            fm_entry("guard", make_entity_decl("guard", "Guard", vec![])),
        ])),
        vec![
            entity_speech("guard", "Halt!"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    if let ContentNode::EntitySpeech(speech) = &linked.graph.nodes["test.urd.md"].ast.content[0] {
        assert!(speech.annotation.is_some());
    }
}

// ═══════════════════════════════════════════════════════════════════
// Additional Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn resolve_exhaustion_check() {
    let ast = make_file_ast(
        "tavern.urd.md",
        None,
        vec![
            section("topics"),
            exhaustion_check("topics"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    if let ContentNode::Condition(cond) = &linked.graph.nodes["tavern.urd.md"].ast.content[1] {
        if let ConditionExpr::ExhaustionCheck(ec) = &cond.expr {
            assert!(ec.annotation.is_some());
            assert_eq!(ec.annotation.as_ref().unwrap().resolved_section, Some("tavern/topics".to_string()));
        }
    }
}

#[test]
fn resolve_set_effect() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Guard", make_type_def("Guard", vec![], vec![
                make_property("mood", "string"),
            ])),
            fm_entry("guard", make_entity_decl("guard", "Guard", vec![])),
        ])),
        vec![
            set_effect("@guard.mood", "happy"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    if let ContentNode::Effect(eff) = &linked.graph.nodes["test.urd.md"].ast.content[0] {
        assert!(eff.annotation.is_some());
        let ann = eff.annotation.as_ref().unwrap();
        assert_eq!(ann.resolved_entity, Some("guard".to_string()));
        assert_eq!(ann.resolved_property, Some("mood".to_string()));
    }
}

#[test]
fn resolve_move_effect() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Item", make_type_def("Item", vec![], vec![])),
            fm_entry("Container", make_type_def("Container", vec![], vec![])),
            fm_entry("key", make_entity_decl("key", "Item", vec![])),
            fm_entry("chest", make_entity_decl("chest", "Container", vec![])),
        ])),
        vec![
            move_effect("key", "chest"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    if let ContentNode::Effect(eff) = &linked.graph.nodes["test.urd.md"].ast.content[0] {
        assert!(eff.annotation.is_some());
        assert_eq!(eff.annotation.as_ref().unwrap().resolved_entity, Some("key".to_string()));
    }
}

#[test]
fn resolve_destroy_effect() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Item", make_type_def("Item", vec![], vec![])),
            fm_entry("key", make_entity_decl("key", "Item", vec![])),
        ])),
        vec![
            destroy_effect("key"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    if let ContentNode::Effect(eff) = &linked.graph.nodes["test.urd.md"].ast.content[0] {
        assert!(eff.annotation.is_some());
        assert_eq!(eff.annotation.as_ref().unwrap().resolved_entity, Some("key".to_string()));
    }
}

#[test]
fn resolve_containment_check() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Item", make_type_def("Item", vec![], vec![])),
            fm_entry("Container", make_type_def("Container", vec![], vec![])),
            fm_entry("key", make_entity_decl("key", "Item", vec![])),
            fm_entry("chest", make_entity_decl("chest", "Container", vec![])),
        ])),
        vec![
            containment_check("key", "chest", false),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    if let ContentNode::Condition(cond) = &linked.graph.nodes["test.urd.md"].ast.content[0] {
        if let ConditionExpr::ContainmentCheck(cc) = &cond.expr {
            assert!(cc.annotation.is_some());
            assert_eq!(cc.annotation.as_ref().unwrap().resolved_entity, Some("key".to_string()));
        }
    }
}

#[test]
fn resolve_unknown_type_with_suggestion() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Guard", make_type_def("Guard", vec![], vec![])),
            fm_entry("guard", make_entity_decl("guard", "Gurad", vec![])),
        ])),
        Vec::new(),
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let _linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD307"));
    assert!(has_suggestion(&diag, "URD307"));
}

#[test]
fn resolve_unknown_property() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Guard", make_type_def("Guard", vec![], vec![
                make_property("mood", "string"),
            ])),
            fm_entry("guard", make_entity_decl("guard", "Guard", vec![
                ("nonexistent", Scalar::String("value".to_string())),
            ])),
        ])),
        Vec::new(),
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let _linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD308"));
}

#[test]
fn resolve_unresolved_explicit_exit() {
    let ast = make_file_ast(
        "test.urd.md",
        None,
        vec![
            location("Cell"),
            exit_jump("west"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let _linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD311"));
}

#[test]
fn collect_sequence_and_phase() {
    let ast = make_file_ast(
        "test.urd.md",
        None,
        vec![
            sequence_heading("Morning Routine"),
            phase_heading("Wake Up", false),
            phase_heading("Breakfast", true),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    assert!(linked.symbol_table.sequences.contains_key("morning-routine"));
    let seq = &linked.symbol_table.sequences["morning-routine"];
    assert_eq!(seq.phases.len(), 2);
    assert_eq!(seq.phases[0].id, "wake-up");
    assert_eq!(seq.phases[0].advance, "manual");
    assert_eq!(seq.phases[1].id, "breakfast");
    assert_eq!(seq.phases[1].advance, "auto");
}

#[test]
fn edit_distance_basic() {
    // Verify the edit distance function directly.
    assert_eq!(urd_compiler::link::edit_distance("guard", "gaurd"), 2);
    assert_eq!(urd_compiler::link::edit_distance("guard", "guard"), 0);
    assert_eq!(urd_compiler::link::edit_distance("guard", "gurd"), 1);
    assert_eq!(urd_compiler::link::edit_distance("", "abc"), 3);
    assert_eq!(urd_compiler::link::edit_distance("abc", ""), 3);
}

// ═══════════════════════════════════════════════════════════════════
// 8. Compliance Fixes – Regression Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn world_config_start_and_entry_stored() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("start", Scalar::String("Grand Hall".to_string())),
                ("entry", Scalar::String("morning-routine".to_string())),
            ])),
        ])),
        vec![
            location("Grand Hall"),
            sequence_heading("Morning Routine"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors());
    assert_eq!(linked.symbol_table.world_start, Some("grand-hall".to_string()));
    assert_eq!(linked.symbol_table.world_entry, Some("morning-routine".to_string()));
}

#[test]
fn world_config_unresolved_not_stored() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("world", make_world_block(vec![
                ("start", Scalar::String("Nonexistent".to_string())),
            ])),
        ])),
        Vec::new(),
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert_eq!(linked.symbol_table.world_start, None);
}

#[test]
fn duplicate_sequence_detected() {
    let ast = make_file_ast(
        "test.urd.md",
        None,
        vec![
            sequence_heading("Morning Routine"),
            phase_heading("Phase A", false),
            sequence_heading("Morning Routine"),
            phase_heading("Phase B", false),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD313")); // Duplicate sequence
    assert_eq!(linked.symbol_table.duplicates.iter().filter(|d| d.namespace == "sequences").count(), 1);
    // First declaration wins — only one entry in the sequences map.
    assert_eq!(linked.symbol_table.sequences.len(), 1);
    let seq = &linked.symbol_table.sequences["morning-routine"];
    // Both phases attach to the original sequence since current_sequence_id is set
    // to the same ID after the duplicate heading.
    assert_eq!(seq.phases.len(), 2);
    assert_eq!(seq.phases[0].id, "phase-a");
    assert_eq!(seq.phases[1].id, "phase-b");
}

#[test]
fn duplicate_rule_detected() {
    let ast = make_file_ast(
        "test.urd.md",
        None,
        vec![
            ContentNode::RuleBlock(RuleBlock {
                name: "patrol".to_string(),
                actor: "guard".to_string(),
                trigger: "idle".to_string(),
                select: None,
                where_clauses: Vec::new(),
                effects: Vec::new(),
                span: span("test.urd.md", 10),
            }),
            ContentNode::RuleBlock(RuleBlock {
                name: "patrol".to_string(),
                actor: "knight".to_string(),
                trigger: "idle".to_string(),
                select: None,
                where_clauses: Vec::new(),
                effects: Vec::new(),
                span: span("test.urd.md", 20),
            }),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD302")); // Duplicate rule
    assert_eq!(linked.symbol_table.duplicates.iter().filter(|d| d.namespace == "rules").count(), 1);
    // First declaration wins.
    let rule = &linked.symbol_table.rules["patrol"];
    assert_eq!(rule.actor, "guard");
}

#[test]
fn entity_presence_outside_location_returns_early() {
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Item", make_type_def("Item", vec![], vec![])),
            fm_entry("key", make_entity_decl("key", "Item", vec![])),
        ])),
        vec![
            // EntityPresence before any location heading.
            entity_presence(vec!["key"]),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD314"));
    // The entity should NOT be added to any location's contains.
    for (_id, loc) in &linked.symbol_table.locations {
        assert!(loc.contains.is_empty());
    }
}

#[test]
fn type_not_visible_uses_suggestion_field() {
    let ast_types = make_file_ast(
        "types.urd.md",
        Some(Frontmatter {
            entries: vec![FrontmatterEntry {
                key: "Guard".to_string(),
                value: make_type_def_in("Guard", "types.urd.md", 2, vec![], vec![]),
                span: span("types.urd.md", 1),
            }],
            span: span("types.urd.md", 1),
        }),
        Vec::new(),
    );
    let ast_main = make_file_ast(
        "main.urd.md",
        Some(Frontmatter {
            entries: vec![FrontmatterEntry {
                key: "guard".to_string(),
                value: make_entity_decl_in("guard", "Guard", "main.urd.md", 4, vec![]),
                span: span("main.urd.md", 1),
            }],
            span: span("main.urd.md", 1),
        }),
        Vec::new(),
    );
    // No import edge: types.urd.md is NOT imported by main.urd.md.
    let cu = two_file_cu_no_import(ast_types, ast_main);
    let mut diag = DiagnosticCollector::new();
    let _linked = link::link(cu, &mut diag);

    assert!(has_error(&diag, "URD301"));
    // The not-visible hint should be in the suggestion field, not the primary message.
    let diag_301 = diag.all().iter().find(|d| d.code == "URD301").unwrap();
    assert!(diag_301.suggestion.is_some());
    assert!(diag_301.suggestion.as_ref().unwrap().contains("not imported"));
    // Primary message should be concise.
    assert!(diag_301.message.contains("Unresolved type reference"));
    assert!(!diag_301.message.contains("not imported"));
}

// ── Container / Destination Resolution Tests ──

#[test]
fn containment_check_player_keyword() {
    // ? @key in player → ContainerKind::KeywordPlayer, no URD301
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Key", make_type_def("Key", vec!["portable"], vec![])),
            fm_entry("key", make_entity_decl("key", "Key", vec![])),
        ])),
        vec![
            containment_check("key", "player", false),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors(), "Expected no errors, got: {:?}", diag.all());
    let node = linked.graph.nodes.get("test.urd.md").unwrap();
    if let ContentNode::Condition(cond) = &node.ast.content[0] {
        if let ConditionExpr::ContainmentCheck(cc) = &cond.expr {
            let ann = cc.annotation.as_ref().expect("annotation should be set");
            assert_eq!(ann.container_kind, Some(ContainerKind::KeywordPlayer));
            assert_eq!(ann.resolved_entity, Some("key".to_string()));
        } else { panic!("expected ContainmentCheck"); }
    } else { panic!("expected Condition"); }
}

#[test]
fn containment_check_here_keyword() {
    // ? @key in here → ContainerKind::KeywordHere, no URD301
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Key", make_type_def("Key", vec!["portable"], vec![])),
            fm_entry("key", make_entity_decl("key", "Key", vec![])),
        ])),
        vec![
            containment_check("key", "here", false),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors(), "Expected no errors, got: {:?}", diag.all());
    let node = linked.graph.nodes.get("test.urd.md").unwrap();
    if let ContentNode::Condition(cond) = &node.ast.content[0] {
        if let ConditionExpr::ContainmentCheck(cc) = &cond.expr {
            let ann = cc.annotation.as_ref().expect("annotation should be set");
            assert_eq!(ann.container_kind, Some(ContainerKind::KeywordHere));
        } else { panic!("expected ContainmentCheck"); }
    } else { panic!("expected Condition"); }
}

#[test]
fn containment_check_entity_container() {
    // ? @key in @chest → ContainerKind::EntityRef("chest")
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Key", make_type_def("Key", vec!["portable"], vec![])),
            fm_entry("Container", make_type_def("Container", vec!["container"], vec![])),
            fm_entry("key", make_entity_decl("key", "Key", vec![])),
            fm_entry("chest", make_entity_decl("chest", "Container", vec![])),
        ])),
        vec![
            containment_check("key", "chest", false),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors(), "Expected no errors, got: {:?}", diag.all());
    let node = linked.graph.nodes.get("test.urd.md").unwrap();
    if let ContentNode::Condition(cond) = &node.ast.content[0] {
        if let ConditionExpr::ContainmentCheck(cc) = &cond.expr {
            let ann = cc.annotation.as_ref().expect("annotation should be set");
            assert_eq!(ann.container_kind, Some(ContainerKind::EntityRef("chest".to_string())));
        } else { panic!("expected ContainmentCheck"); }
    } else { panic!("expected Condition"); }
}

#[test]
fn containment_check_location_container() {
    // ? @key in cellar → ContainerKind::LocationRef("cellar")
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Key", make_type_def("Key", vec!["portable"], vec![])),
            fm_entry("key", make_entity_decl("key", "Key", vec![])),
        ])),
        vec![
            location("Cellar"),
            containment_check("key", "cellar", false),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors(), "Expected no errors, got: {:?}", diag.all());
    let node = linked.graph.nodes.get("test.urd.md").unwrap();
    // Content: [location, containment_check]
    if let ContentNode::Condition(cond) = &node.ast.content[1] {
        if let ConditionExpr::ContainmentCheck(cc) = &cond.expr {
            let ann = cc.annotation.as_ref().expect("annotation should be set");
            assert_eq!(ann.container_kind, Some(ContainerKind::LocationRef("cellar".to_string())));
        } else { panic!("expected ContainmentCheck"); }
    } else { panic!("expected Condition"); }
}

#[test]
fn move_player_destination() {
    // > move @key -> player → DestinationKind::KeywordPlayer
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Key", make_type_def("Key", vec!["portable"], vec![])),
            fm_entry("key", make_entity_decl("key", "Key", vec![])),
        ])),
        vec![
            move_effect("key", "player"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors(), "Expected no errors, got: {:?}", diag.all());
    let node = linked.graph.nodes.get("test.urd.md").unwrap();
    if let ContentNode::Effect(eff) = &node.ast.content[0] {
        let ann = eff.annotation.as_ref().expect("annotation should be set");
        assert_eq!(ann.destination_kind, Some(DestinationKind::KeywordPlayer));
        assert_eq!(ann.resolved_entity, Some("key".to_string()));
    } else { panic!("expected Effect"); }
}

#[test]
fn move_location_destination() {
    // > move @key -> cellar → DestinationKind::LocationRef("cellar")
    let ast = make_file_ast(
        "test.urd.md",
        Some(make_frontmatter(vec![
            fm_entry("Key", make_type_def("Key", vec!["portable"], vec![])),
            fm_entry("key", make_entity_decl("key", "Key", vec![])),
        ])),
        vec![
            location("Cellar"),
            move_effect("key", "cellar"),
        ],
    );
    let cu = single_file_cu(ast);
    let mut diag = DiagnosticCollector::new();
    let linked = link::link(cu, &mut diag);

    assert!(!diag.has_errors(), "Expected no errors, got: {:?}", diag.all());
    let node = linked.graph.nodes.get("test.urd.md").unwrap();
    // Content: [location, move_effect]
    if let ContentNode::Effect(eff) = &node.ast.content[1] {
        let ann = eff.annotation.as_ref().expect("annotation should be set");
        assert_eq!(ann.destination_kind, Some(DestinationKind::LocationRef("cellar".to_string())));
    } else { panic!("expected Effect"); }
}
