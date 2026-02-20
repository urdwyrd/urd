/// Phase 5: EMIT — validated ASTs + symbol table to `.urd.json`.
///
/// Input:  Validated ASTs + `SymbolTable` (with zero Error-severity diagnostics)
/// Output: `.urd.json` string
///
/// EMIT runs only when the diagnostic collector contains zero errors.
/// It traverses pre-validated data structures in a fixed, deterministic order.
///
/// Key guarantee: output conforms to JSON Schema, deterministic,
/// `urd: "1"` injected, byte-identical across repeated compilations.
///
/// Diagnostic code range: URD500–URD599

use indexmap::IndexMap;
use serde_json::{Map, Value as Json, Number};

use crate::ast::{
    ConditionExpr, ContainerKind, ContentNode, DestinationKind, EffectType,
    FrontmatterValue, Scalar,
};
use crate::diagnostics::DiagnosticCollector;
use crate::graph::{file_stem, DependencyGraph};
use crate::slugify::slugify;
use crate::symbol_table::{
    PropertyType, SymbolTable, Value, Visibility,
};

/// Emit the compiled `.urd.json` string from the validated world.
///
/// Precondition: `diagnostics.has_errors()` is `false`.
pub fn emit(
    graph: &DependencyGraph,
    symbol_table: &SymbolTable,
    _diagnostics: &mut DiagnosticCollector,
) -> String {
    let ordered = graph.topological_order();
    let ordered_paths: Vec<&str> = ordered.iter().map(|s| s.as_str()).collect();

    let mut root = Map::new();

    // Step 1: world (always present)
    root.insert("world".to_string(), build_world(graph, symbol_table));

    // Step 2: types
    if !symbol_table.types.is_empty() {
        root.insert("types".to_string(), build_types(symbol_table));
    }

    // Step 3: entities
    if !symbol_table.entities.is_empty() {
        root.insert("entities".to_string(), build_entities(symbol_table));
    }

    // Step 4: locations
    if !symbol_table.locations.is_empty() {
        root.insert(
            "locations".to_string(),
            build_locations(graph, &ordered_paths, symbol_table),
        );
    }

    // Step 5: rules
    if !symbol_table.rules.is_empty() {
        root.insert(
            "rules".to_string(),
            build_rules(graph, &ordered_paths, symbol_table),
        );
    }

    // Step 6: actions
    if !symbol_table.actions.is_empty() {
        root.insert(
            "actions".to_string(),
            build_actions(graph, &ordered_paths, symbol_table),
        );
    }

    // Step 7: sequences
    if !symbol_table.sequences.is_empty() {
        root.insert("sequences".to_string(), build_sequences(symbol_table));
    }

    // Step 8: dialogue
    if !symbol_table.sections.is_empty() {
        root.insert(
            "dialogue".to_string(),
            build_dialogue(graph, &ordered_paths, symbol_table),
        );
    }

    let json_value = Json::Object(root);
    let mut output = serde_json::to_string_pretty(&json_value).unwrap();
    output.push('\n');
    output
}

// ── Step 1: World ──

fn build_world(graph: &DependencyGraph, symbol_table: &SymbolTable) -> Json {
    let mut world = Map::new();

    // Find WorldBlock in entry file's frontmatter.
    if let Some(entry_path) = &graph.entry_path {
        if let Some(node) = graph.nodes.get(entry_path.as_str()) {
            if let Some(fm) = &node.ast.frontmatter {
                for entry in &fm.entries {
                    if let FrontmatterValue::WorldBlock(wb) = &entry.value {
                        for (key, val) in &wb.fields {
                            match key.as_str() {
                                "name" => {
                                    let raw = scalar_to_json(val);
                                    let slugified = if let Json::String(s) = &raw {
                                        Json::String(slugify(s))
                                    } else {
                                        raw
                                    };
                                    world.insert(
                                        "name".to_string(),
                                        slugified,
                                    );
                                }
                                // urd is injected below, skip any author value.
                                "urd" => {}
                                "version" => {
                                    world.insert(
                                        "version".to_string(),
                                        scalar_to_json(val),
                                    );
                                }
                                "description" => {
                                    world.insert(
                                        "description".to_string(),
                                        scalar_to_json(val),
                                    );
                                }
                                "author" => {
                                    world.insert(
                                        "author".to_string(),
                                        scalar_to_json(val),
                                    );
                                }
                                "seed" => {
                                    world.insert("seed".to_string(), scalar_to_json(val));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    // Re-insert in fixed key order: name, urd, version, description, author, start, entry, seed.
    let mut ordered = Map::new();
    if let Some(v) = world.remove("name") {
        ordered.insert("name".to_string(), v);
    }
    ordered.insert("urd".to_string(), Json::String("1".to_string()));
    if let Some(v) = world.remove("version") {
        ordered.insert("version".to_string(), v);
    }
    if let Some(v) = world.remove("description") {
        ordered.insert("description".to_string(), v);
    }
    if let Some(v) = world.remove("author") {
        ordered.insert("author".to_string(), v);
    }
    if let Some(start) = &symbol_table.world_start {
        ordered.insert("start".to_string(), Json::String(start.clone()));
    }
    if let Some(entry) = &symbol_table.world_entry {
        ordered.insert("entry".to_string(), Json::String(entry.clone()));
    }
    if let Some(v) = world.remove("seed") {
        ordered.insert("seed".to_string(), v);
    }

    Json::Object(ordered)
}

// ── Step 2: Types ──

fn build_types(symbol_table: &SymbolTable) -> Json {
    let mut types = Map::new();
    for (name, ts) in &symbol_table.types {
        let mut type_obj = Map::new();

        if !ts.traits.is_empty() {
            type_obj.insert(
                "traits".to_string(),
                Json::Array(ts.traits.iter().map(|t| Json::String(t.clone())).collect()),
            );
        }

        if !ts.properties.is_empty() {
            let mut props = Map::new();
            for (prop_name, ps) in &ts.properties {
                props.insert(prop_name.clone(), build_property(ps));
            }
            type_obj.insert("properties".to_string(), Json::Object(props));
        }

        types.insert(name.clone(), Json::Object(type_obj));
    }
    Json::Object(types)
}

fn build_property(ps: &crate::symbol_table::PropertySymbol) -> Json {
    let mut prop = Map::new();

    // type
    prop.insert("type".to_string(), Json::String(format_property_type(&ps.property_type).to_string()));

    // default
    if let Some(default) = &ps.default {
        prop.insert("default".to_string(), value_to_json(default));
    }

    // visibility (omit if visible)
    if ps.visibility == Visibility::Hidden {
        prop.insert("visibility".to_string(), Json::String("hidden".to_string()));
    }

    // description
    if let Some(desc) = &ps.description {
        prop.insert("description".to_string(), Json::String(desc.clone()));
    }

    // values (enum)
    if let Some(values) = &ps.values {
        prop.insert(
            "values".to_string(),
            Json::Array(values.iter().map(|v| Json::String(v.clone())).collect()),
        );
    }

    // min/max
    if let Some(min) = ps.min {
        prop.insert("min".to_string(), number_to_json(min));
    }
    if let Some(max) = ps.max {
        prop.insert("max".to_string(), number_to_json(max));
    }

    // ref_type
    if let Some(rt) = &ps.ref_type {
        prop.insert("ref_type".to_string(), Json::String(rt.clone()));
    }

    Json::Object(prop)
}

// ── Step 3: Entities ──

fn build_entities(symbol_table: &SymbolTable) -> Json {
    let mut entities = Map::new();
    for (id, es) in &symbol_table.entities {
        let mut entity_obj = Map::new();
        entity_obj.insert("type".to_string(), Json::String(es.type_name.clone()));

        if !es.property_overrides.is_empty() {
            let mut props = Map::new();
            for (prop_name, val) in &es.property_overrides {
                props.insert(prop_name.clone(), value_to_json(val));
            }
            entity_obj.insert("properties".to_string(), Json::Object(props));
        }

        entities.insert(id.clone(), Json::Object(entity_obj));
    }
    Json::Object(entities)
}

// ── Step 4: Locations ──

fn build_locations(
    graph: &DependencyGraph,
    ordered_paths: &[&str],
    symbol_table: &SymbolTable,
) -> Json {
    // Collect location descriptions and exit content from AST.
    let mut loc_descriptions: IndexMap<String, Vec<String>> = IndexMap::new();
    let mut exit_content: IndexMap<(String, String), ExitContent> = IndexMap::new();

    for file_path in ordered_paths {
        let node = match graph.nodes.get(*file_path) {
            Some(n) => n,
            None => continue,
        };

        let mut current_loc_id: Option<String> = None;
        let mut desc_buffer: Vec<String> = Vec::new();
        let mut in_desc_zone = false;

        for content in &node.ast.content {
            match content {
                ContentNode::LocationHeading(loc) => {
                    // Flush previous location's description.
                    if let Some(loc_id) = &current_loc_id {
                        if !desc_buffer.is_empty() {
                            loc_descriptions
                                .entry(loc_id.clone())
                                .or_default()
                                .extend(desc_buffer.drain(..));
                        }
                    }
                    let id = slugify(&loc.display_name);
                    current_loc_id = Some(id);
                    in_desc_zone = true;
                    desc_buffer.clear();
                }
                ContentNode::Prose(prose) if in_desc_zone => {
                    let trimmed = prose.text.trim().to_string();
                    if !trimmed.is_empty() {
                        desc_buffer.push(trimmed);
                    }
                }
                ContentNode::ExitDeclaration(exit) => {
                    in_desc_zone = false;
                    if let Some(loc_id) = &current_loc_id {
                        let ec = collect_exit_content(&exit.children, symbol_table);
                        exit_content.insert(
                            (loc_id.clone(), exit.direction.clone()),
                            ec,
                        );
                    }
                }
                ContentNode::EntityPresence(_)
                | ContentNode::SectionLabel(_)
                | ContentNode::Choice(_)
                | ContentNode::SequenceHeading(_)
                | ContentNode::PhaseHeading(_) => {
                    in_desc_zone = false;
                }
                _ => {}
            }
        }
        // Flush last location's description.
        if let Some(loc_id) = &current_loc_id {
            if !desc_buffer.is_empty() {
                loc_descriptions
                    .entry(loc_id.clone())
                    .or_default()
                    .extend(desc_buffer.drain(..));
            }
        }
    }

    // Build location JSON objects.
    let mut locations = Map::new();
    for (id, ls) in &symbol_table.locations {
        let mut loc_obj = Map::new();

        // description
        if let Some(prose_blocks) = loc_descriptions.get(id) {
            if !prose_blocks.is_empty() {
                loc_obj.insert(
                    "description".to_string(),
                    Json::String(prose_blocks.join("\n\n")),
                );
            }
        }

        // contains
        if !ls.contains.is_empty() {
            loc_obj.insert(
                "contains".to_string(),
                Json::Array(
                    ls.contains
                        .iter()
                        .map(|e| Json::String(strip_at(e)))
                        .collect(),
                ),
            );
        }

        // exits
        if !ls.exits.is_empty() {
            let mut exits = Map::new();
            for (direction, es) in &ls.exits {
                let mut exit_obj = Map::new();

                // to
                if let Some(dest) = &es.resolved_destination {
                    exit_obj.insert("to".to_string(), Json::String(dest.clone()));
                }

                // condition, blocked_message, effects from AST
                if let Some(ec) = exit_content.get(&(id.clone(), direction.clone())) {
                    if let Some(cond_str) = &ec.condition {
                        exit_obj.insert("condition".to_string(), Json::String(cond_str.clone()));
                    }
                    if let Some(msg) = &ec.blocked_message {
                        exit_obj.insert("blocked_message".to_string(), Json::String(msg.clone()));
                    }
                    if !ec.effects.is_empty() {
                        exit_obj.insert("effects".to_string(), Json::Array(ec.effects.clone()));
                    }
                }

                exits.insert(direction.clone(), Json::Object(exit_obj));
            }
            loc_obj.insert("exits".to_string(), Json::Object(exits));
        }

        locations.insert(id.clone(), Json::Object(loc_obj));
    }
    Json::Object(locations)
}

struct ExitContent {
    condition: Option<String>,
    blocked_message: Option<String>,
    effects: Vec<Json>,
}

fn collect_exit_content(children: &[ContentNode], symbol_table: &SymbolTable) -> ExitContent {
    let mut condition = None;
    let mut blocked_message = None;
    let mut effects = Vec::new();

    for child in children {
        match child {
            ContentNode::Condition(cond) => {
                condition = Some(lower_condition(&cond.expr, symbol_table));
            }
            ContentNode::BlockedMessage(bm) => {
                blocked_message = Some(bm.text.clone());
            }
            ContentNode::Effect(eff) => {
                effects.push(lower_effect(&eff.effect_type, &eff.annotation, symbol_table));
            }
            _ => {}
        }
    }

    ExitContent {
        condition,
        blocked_message,
        effects,
    }
}

// ── Step 5: Rules ──

fn build_rules(
    graph: &DependencyGraph,
    ordered_paths: &[&str],
    symbol_table: &SymbolTable,
) -> Json {
    // Collect RuleBlock AST nodes by name.
    let mut rule_blocks: IndexMap<String, &crate::ast::RuleBlock> = IndexMap::new();
    for file_path in ordered_paths {
        let node = match graph.nodes.get(*file_path) {
            Some(n) => n,
            None => continue,
        };
        for content in &node.ast.content {
            if let ContentNode::RuleBlock(rb) = content {
                rule_blocks.insert(rb.name.clone(), rb);
            }
        }
    }

    let mut rules = Map::new();
    for (name, rs) in &symbol_table.rules {
        let mut rule_obj = Map::new();

        // actor
        if !rs.actor.is_empty() {
            rule_obj.insert(
                "actor".to_string(),
                Json::String(strip_at(&rs.actor)),
            );
        }

        // trigger
        rule_obj.insert("trigger".to_string(), Json::String(rs.trigger.clone()));

        // conditions from RuleBlock.where_clauses
        if let Some(rb) = rule_blocks.get(name) {
            if !rb.where_clauses.is_empty() {
                let conds: Vec<Json> = rb
                    .where_clauses
                    .iter()
                    .map(|c| Json::String(lower_condition(c, symbol_table)))
                    .collect();
                rule_obj.insert("conditions".to_string(), Json::Array(conds));
            }
        }

        // select
        if let Some(sel) = &rs.select {
            let mut sel_obj = Map::new();
            sel_obj.insert(
                "from".to_string(),
                Json::Array(
                    sel.from
                        .iter()
                        .map(|e| Json::String(strip_at(e)))
                        .collect(),
                ),
            );
            sel_obj.insert("as".to_string(), Json::String(sel.variable.clone()));
            if !sel.where_clauses.is_empty() {
                let where_conds: Vec<Json> = sel
                    .where_clauses
                    .iter()
                    .map(|c| Json::String(lower_condition(c, symbol_table)))
                    .collect();
                sel_obj.insert("where".to_string(), Json::Array(where_conds));
            }
            rule_obj.insert("select".to_string(), Json::Object(sel_obj));
        }

        // effects from RuleBlock.effects
        if let Some(rb) = rule_blocks.get(name) {
            if !rb.effects.is_empty() {
                let effs: Vec<Json> = rb
                    .effects
                    .iter()
                    .map(|e| lower_effect(&e.effect_type, &e.annotation, symbol_table))
                    .collect();
                rule_obj.insert("effects".to_string(), Json::Array(effs));
            }
        }

        rules.insert(name.clone(), Json::Object(rule_obj));
    }
    Json::Object(rules)
}

// ── Step 6: Actions ──

fn build_actions(
    graph: &DependencyGraph,
    ordered_paths: &[&str],
    symbol_table: &SymbolTable,
) -> Json {
    // Correlate choice compiled_ids to Choice AST nodes.
    let mut choice_nodes: IndexMap<String, &crate::ast::Choice> = IndexMap::new();
    for file_path in ordered_paths {
        let node = match graph.nodes.get(*file_path) {
            Some(n) => n,
            None => continue,
        };
        let stem = file_stem(file_path);
        let mut current_section_id: Option<String> = None;

        for content in &node.ast.content {
            match content {
                ContentNode::SectionLabel(sl) => {
                    current_section_id = Some(format!("{}/{}", stem, sl.name));
                }
                ContentNode::LocationHeading(_) => {
                    current_section_id = None;
                }
                ContentNode::Choice(choice) => {
                    if let Some(section_id) = &current_section_id {
                        collect_choice_nodes(choice, section_id, &mut choice_nodes);
                    }
                }
                _ => {}
            }
        }
    }

    let mut actions = Map::new();
    for (id, as_) in &symbol_table.actions {
        let mut action_obj = Map::new();

        // target
        if let Some(target) = &as_.target {
            action_obj.insert(
                "target".to_string(),
                Json::String(strip_at(target)),
            );
        }

        // target_type
        if let Some(tt) = &as_.target_type {
            action_obj.insert("target_type".to_string(), Json::String(tt.clone()));
        }

        // conditions and effects from the Choice AST node
        if let Some(choice) = choice_nodes.get(id) {
            let (conds, effects) = collect_choice_conditions_effects(choice, symbol_table);
            if let Some(c) = conds {
                action_obj.insert("conditions".to_string(), c);
            }
            action_obj.insert("effects".to_string(), Json::Array(effects));
        }

        actions.insert(id.clone(), Json::Object(action_obj));
    }
    Json::Object(actions)
}

fn collect_choice_nodes<'a>(
    choice: &'a crate::ast::Choice,
    section_id: &str,
    map: &mut IndexMap<String, &'a crate::ast::Choice>,
) {
    let slug = slugify(&choice.label);
    if slug.is_empty() {
        return;
    }
    let choice_id = format!("{}/{}", section_id, slug);
    map.insert(choice_id.clone(), choice);

    // Recurse into nested choices.
    for child in &choice.content {
        if let ContentNode::Choice(sub) = child {
            collect_choice_nodes(sub, section_id, map);
        }
    }
}

// ── Step 7: Sequences ──

fn build_sequences(symbol_table: &SymbolTable) -> Json {
    let mut sequences = Map::new();
    for (id, ss) in &symbol_table.sequences {
        let mut seq_obj = Map::new();

        let phases: Vec<Json> = ss.phases.iter().map(|ps| {
            let mut phase_obj = Map::new();

            phase_obj.insert("id".to_string(), Json::String(ps.id.clone()));

            // auto (omit if false)
            if ps.advance == "auto" {
                phase_obj.insert("auto".to_string(), Json::Bool(true));
            }

            // action
            if let Some(action) = &ps.action {
                phase_obj.insert("action".to_string(), Json::String(action.clone()));
            }

            // actions
            if let Some(actions_list) = &ps.actions {
                phase_obj.insert(
                    "actions".to_string(),
                    Json::Array(actions_list.iter().map(|a| Json::String(a.clone())).collect()),
                );
            }

            // rule
            if let Some(rule) = &ps.rule {
                phase_obj.insert("rule".to_string(), Json::String(rule.clone()));
            }

            // advance
            let advance_str = format_advance(&ps.advance);
            phase_obj.insert("advance".to_string(), Json::String(advance_str));

            Json::Object(phase_obj)
        }).collect();

        seq_obj.insert("phases".to_string(), Json::Array(phases));

        sequences.insert(id.clone(), Json::Object(seq_obj));
    }
    Json::Object(sequences)
}

fn format_advance(advance: &str) -> String {
    // The advance string is already the correct value for most modes.
    // For on_condition, ensure the expression part is space-free.
    if let Some(expr) = advance.strip_prefix("on_condition ") {
        // Make the expression space-free: remove spaces around operators.
        let space_free = expr
            .replace(" == ", "==")
            .replace(" != ", "!=")
            .replace(" < ", "<")
            .replace(" > ", ">")
            .replace(" <= ", "<=")
            .replace(" >= ", ">=");
        format!("on_condition {}", space_free)
    } else {
        advance.to_string()
    }
}

// ── Step 8: Dialogue ──

fn build_dialogue(
    graph: &DependencyGraph,
    ordered_paths: &[&str],
    symbol_table: &SymbolTable,
) -> Json {
    // Collect section content from AST.
    let mut section_data: IndexMap<String, SectionData> = IndexMap::new();

    for file_path in ordered_paths {
        let node = match graph.nodes.get(*file_path) {
            Some(n) => n,
            None => continue,
        };
        let stem = file_stem(file_path);
        let mut current_section_id: Option<String> = None;
        let mut current_nodes: Vec<&ContentNode> = Vec::new();

        for content in &node.ast.content {
            match content {
                ContentNode::SectionLabel(sl) => {
                    // Flush previous section.
                    if let Some(sec_id) = &current_section_id {
                        section_data.insert(
                            sec_id.clone(),
                            build_section_data(&current_nodes, symbol_table),
                        );
                    }
                    current_section_id = Some(format!("{}/{}", stem, sl.name));
                    current_nodes.clear();
                }
                ContentNode::LocationHeading(_) => {
                    // Flush previous section.
                    if let Some(sec_id) = &current_section_id {
                        section_data.insert(
                            sec_id.clone(),
                            build_section_data(&current_nodes, symbol_table),
                        );
                    }
                    current_section_id = None;
                    current_nodes.clear();
                }
                _ => {
                    if current_section_id.is_some() {
                        current_nodes.push(content);
                    }
                }
            }
        }
        // Flush last section in file.
        if let Some(sec_id) = &current_section_id {
            section_data.insert(
                sec_id.clone(),
                build_section_data(&current_nodes, symbol_table),
            );
        }
    }

    // Build dialogue JSON objects.
    let mut dialogue = Map::new();
    for (id, ss) in &symbol_table.sections {
        let mut sec_obj = Map::new();

        // id (required)
        sec_obj.insert("id".to_string(), Json::String(id.clone()));

        if let Some(sd) = section_data.get(id) {
            // prompt
            if let Some((speaker, text)) = &sd.prompt {
                let mut speech = Map::new();
                speech.insert("speaker".to_string(), Json::String(strip_at(speaker)));
                speech.insert("text".to_string(), Json::String(text.clone()));
                sec_obj.insert("prompt".to_string(), Json::Object(speech));
            }

            // description
            if let Some(desc) = &sd.description {
                sec_obj.insert("description".to_string(), Json::String(desc.clone()));
            }

            // conditions
            if let Some(conds) = &sd.conditions {
                sec_obj.insert("conditions".to_string(), conds.clone());
            }

            // choices
            if !sd.choices.is_empty() {
                let choices_json: Vec<Json> = sd
                    .choices
                    .iter()
                    .zip(ss.choices.iter())
                    .map(|(cd, cs)| build_choice_json(cd, cs, symbol_table))
                    .collect();
                sec_obj.insert("choices".to_string(), Json::Array(choices_json));
            }

            // on_exhausted
            if let Some(exhausted) = &sd.on_exhausted {
                let mut ex_obj = Map::new();
                if let Some(speaker) = &exhausted.speaker {
                    ex_obj.insert("speaker".to_string(), Json::String(strip_at(speaker)));
                }
                ex_obj.insert("text".to_string(), Json::String(exhausted.text.clone()));
                if let Some(goto) = &exhausted.goto {
                    ex_obj.insert("goto".to_string(), Json::String(goto.clone()));
                }
                sec_obj.insert("on_exhausted".to_string(), Json::Object(ex_obj));
            }
        }

        dialogue.insert(id.clone(), Json::Object(sec_obj));
    }
    Json::Object(dialogue)
}

struct SectionData {
    prompt: Option<(String, String)>, // (speaker, text)
    description: Option<String>,
    conditions: Option<Json>,
    choices: Vec<ChoiceData>,
    on_exhausted: Option<ExhaustedData>,
}

struct ChoiceData {
    label: String,
    conditions: Option<Json>,
    response: Option<(String, String)>, // (speaker, text)
    effects: Vec<Json>,
    goto: Option<String>,
    nested_choices: Vec<ChoiceData>,
}

struct ExhaustedData {
    text: String,
    speaker: Option<String>,
    goto: Option<String>,
}

fn build_section_data(
    nodes: &[&ContentNode],
    symbol_table: &SymbolTable,
) -> SectionData {
    // Partition into Region A, Region B, Region C.
    let mut first_choice_idx = None;
    let mut last_choice_idx = None;

    for (i, node) in nodes.iter().enumerate() {
        if matches!(node, ContentNode::Choice(_)) {
            if first_choice_idx.is_none() {
                first_choice_idx = Some(i);
            }
            last_choice_idx = Some(i);
        }
    }

    let region_a = match first_choice_idx {
        Some(idx) => &nodes[..idx],
        None => nodes,
    };

    let region_c = match last_choice_idx {
        Some(idx) if idx + 1 < nodes.len() => &nodes[idx + 1..],
        _ => &[],
    };

    // Region A: extract prompt, description, conditions.
    let mut prompt: Option<(String, String)> = None;
    let mut prose_blocks: Vec<String> = Vec::new();
    let mut and_conditions: Vec<String> = Vec::new();
    let mut or_conditions: Option<Vec<String>> = None;

    for node in region_a {
        match node {
            ContentNode::EntitySpeech(es) => {
                if prompt.is_none() {
                    prompt = Some((es.entity_ref.clone(), es.text.clone()));
                } else {
                    // Subsequent speech goes into description.
                    let text = format!("{}: {}", es.entity_ref, es.text);
                    prose_blocks.push(text.trim().to_string());
                }
            }
            ContentNode::Prose(p) => {
                let trimmed = p.text.trim().to_string();
                if !trimmed.is_empty() {
                    prose_blocks.push(trimmed);
                }
            }
            ContentNode::StageDirection(sd) => {
                let text = format!("{} {}", sd.entity_ref, sd.text);
                prose_blocks.push(text.trim().to_string());
            }
            ContentNode::Condition(cond) => {
                and_conditions.push(lower_condition(&cond.expr, symbol_table));
            }
            ContentNode::OrConditionBlock(or) => {
                let or_conds: Vec<String> = or
                    .conditions
                    .iter()
                    .map(|c| lower_condition(c, symbol_table))
                    .collect();
                or_conditions = Some(or_conds);
            }
            _ => {}
        }
    }

    let description = if prose_blocks.is_empty() {
        None
    } else {
        Some(prose_blocks.join("\n\n"))
    };

    let conditions = build_conditions_json(&and_conditions, &or_conditions);

    // Region B: extract choices.
    let choices: Vec<ChoiceData> = nodes
        .iter()
        .filter_map(|n| {
            if let ContentNode::Choice(c) = n {
                Some(build_choice_data(c, symbol_table))
            } else {
                None
            }
        })
        .collect();

    // Region C: extract on_exhausted.
    let on_exhausted = build_exhausted_data(region_c, symbol_table);

    SectionData {
        prompt,
        description,
        conditions,
        choices,
        on_exhausted,
    }
}

fn build_choice_data(
    choice: &crate::ast::Choice,
    symbol_table: &SymbolTable,
) -> ChoiceData {
    let mut and_conditions: Vec<String> = Vec::new();
    let mut or_conditions: Option<Vec<String>> = None;
    let mut response: Option<(String, String)> = None;
    let mut effects: Vec<Json> = Vec::new();
    let mut goto: Option<String> = None;
    let mut nested_choices: Vec<ChoiceData> = Vec::new();

    for child in &choice.content {
        match child {
            ContentNode::Condition(cond) => {
                and_conditions.push(lower_condition(&cond.expr, symbol_table));
            }
            ContentNode::OrConditionBlock(or) => {
                let or_conds: Vec<String> = or
                    .conditions
                    .iter()
                    .map(|c| lower_condition(c, symbol_table))
                    .collect();
                or_conditions = Some(or_conds);
            }
            ContentNode::EntitySpeech(es) => {
                if response.is_none() {
                    response = Some((es.entity_ref.clone(), es.text.clone()));
                }
            }
            ContentNode::Effect(eff) => {
                effects.push(lower_effect(&eff.effect_type, &eff.annotation, symbol_table));
            }
            ContentNode::Jump(jump) => {
                // Only emit goto for section jumps (not "end", not target-directed).
                if choice.target.is_none() && choice.target_type.is_none() {
                    if jump.target != "end" {
                        if let Some(ann) = &jump.annotation {
                            if let Some(sec_id) = &ann.resolved_section {
                                goto = Some(sec_id.clone());
                            }
                        }
                    }
                }
            }
            ContentNode::Choice(sub) => {
                nested_choices.push(build_choice_data(sub, symbol_table));
            }
            _ => {}
        }
    }

    let conditions = build_conditions_json(&and_conditions, &or_conditions);

    ChoiceData {
        label: choice.label.clone(),
        conditions,
        response,
        effects,
        goto,
        nested_choices,
    }
}

fn build_choice_json(
    cd: &ChoiceData,
    cs: &crate::symbol_table::ChoiceSymbol,
    symbol_table: &SymbolTable,
) -> Json {
    let mut choice_obj = Map::new();

    // id
    choice_obj.insert("id".to_string(), Json::String(cs.compiled_id.clone()));

    // label
    choice_obj.insert("label".to_string(), Json::String(cs.label.clone()));

    // sticky
    choice_obj.insert("sticky".to_string(), Json::Bool(cs.sticky));

    // conditions
    if let Some(conds) = &cd.conditions {
        choice_obj.insert("conditions".to_string(), conds.clone());
    }

    // response
    if let Some((speaker, text)) = &cd.response {
        let mut speech = Map::new();
        speech.insert("speaker".to_string(), Json::String(strip_at(speaker)));
        speech.insert("text".to_string(), Json::String(text.clone()));
        choice_obj.insert("response".to_string(), Json::Object(speech));
    }

    // effects
    if !cd.effects.is_empty() {
        choice_obj.insert("effects".to_string(), Json::Array(cd.effects.clone()));
    }

    // goto
    if let Some(goto) = &cd.goto {
        choice_obj.insert("goto".to_string(), Json::String(goto.clone()));
    }

    // nested choices
    if !cd.nested_choices.is_empty() {
        // Look up nested ChoiceSymbols from the section.
        // For nested choices, we need to find the corresponding ChoiceSymbols.
        // The nested choices follow the parent in the section's choices list,
        // matched by compiled_id prefix.
        let nested_json: Vec<Json> = cd
            .nested_choices
            .iter()
            .filter_map(|ncd| {
                find_nested_choice_symbol(cs, &ncd.label, symbol_table)
                    .map(|ncs| build_choice_json(ncd, ncs, symbol_table))
            })
            .collect();
        if !nested_json.is_empty() {
            choice_obj.insert("choices".to_string(), Json::Array(nested_json));
        }
    }

    Json::Object(choice_obj)
}

fn find_nested_choice_symbol<'a>(
    parent_cs: &crate::symbol_table::ChoiceSymbol,
    nested_label: &str,
    symbol_table: &'a SymbolTable,
) -> Option<&'a crate::symbol_table::ChoiceSymbol> {
    // Extract section_id from parent's compiled_id (everything before the last '/').
    let section_id = parent_cs.compiled_id.rsplitn(2, '/').nth(1)?;
    let expected_id = format!("{}/{}", section_id, slugify(nested_label));
    symbol_table
        .sections
        .get(section_id)
        .and_then(|ss| ss.choices.iter().find(|c| c.compiled_id == expected_id))
}

fn build_exhausted_data(
    region_c: &[&ContentNode],
    _symbol_table: &SymbolTable,
) -> Option<ExhaustedData> {
    if region_c.is_empty() {
        return None;
    }

    let mut text_parts: Vec<String> = Vec::new();
    let mut speaker: Option<String> = None;
    let mut goto: Option<String> = None;

    for node in region_c {
        match node {
            ContentNode::EntitySpeech(es) => {
                if speaker.is_none() && text_parts.is_empty() {
                    speaker = Some(es.entity_ref.clone());
                    text_parts.push(es.text.clone());
                }
            }
            ContentNode::Prose(p) => {
                let trimmed = p.text.trim().to_string();
                if !trimmed.is_empty() {
                    text_parts.push(trimmed);
                }
            }
            ContentNode::StageDirection(sd) => {
                let text = format!("{} {}", sd.entity_ref, sd.text);
                text_parts.push(text.trim().to_string());
            }
            ContentNode::Jump(jump) => {
                if jump.target != "end" {
                    if let Some(ann) = &jump.annotation {
                        if let Some(sec_id) = &ann.resolved_section {
                            goto = Some(sec_id.clone());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if text_parts.is_empty() {
        return None;
    }

    Some(ExhaustedData {
        text: text_parts.join("\n\n"),
        speaker,
        goto,
    })
}

// ── Condition Lowering ──

fn lower_condition(expr: &ConditionExpr, symbol_table: &SymbolTable) -> String {
    match expr {
        ConditionExpr::PropertyComparison(pc) => {
            format!(
                "{}.{} {} {}",
                strip_at(&pc.entity_ref),
                pc.property,
                pc.operator,
                pc.value,
            )
        }
        ConditionExpr::ContainmentCheck(cc) => {
            let entity = strip_at(&cc.entity_ref);
            let op = if cc.negated { "!=" } else { "==" };
            let container = resolve_container(&cc.annotation, &cc.container_ref, symbol_table);
            format!("{}.container {} {}", entity, op, container)
        }
        ConditionExpr::ExhaustionCheck(ec) => {
            let section_id = ec
                .annotation
                .as_ref()
                .and_then(|a| a.resolved_section.as_ref())
                .cloned()
                .unwrap_or_else(|| ec.section_name.clone());
            format!("{}.exhausted", section_id)
        }
    }
}

fn lower_condition_space_free(expr: &ConditionExpr, symbol_table: &SymbolTable) -> String {
    let s = lower_condition(expr, symbol_table);
    s.replace(" == ", "==")
        .replace(" != ", "!=")
        .replace(" < ", "<")
        .replace(" > ", ">")
        .replace(" <= ", "<=")
        .replace(" >= ", ">=")
}

fn resolve_container(
    annotation: &Option<crate::ast::Annotation>,
    _raw: &str,
    _symbol_table: &SymbolTable,
) -> String {
    if let Some(ann) = annotation {
        if let Some(ck) = &ann.container_kind {
            return match ck {
                ContainerKind::KeywordPlayer => "player".to_string(),
                ContainerKind::KeywordHere => "player.container".to_string(),
                ContainerKind::EntityRef(id) => strip_at(id),
                ContainerKind::LocationRef(id) => id.clone(),
            };
        }
    }
    // Fallback: use raw ref (shouldn't happen if LINK ran).
    strip_at(_raw)
}

// ── Effect Lowering ──

fn lower_effect(
    effect_type: &EffectType,
    annotation: &Option<crate::ast::Annotation>,
    symbol_table: &SymbolTable,
) -> Json {
    match effect_type {
        EffectType::Set {
            target_prop: _,
            operator,
            value_expr,
        } => {
            let ann = annotation.as_ref();
            let entity_id = ann
                .and_then(|a| a.resolved_entity.as_ref())
                .map(|e| strip_at(e))
                .unwrap_or_default();
            let prop_name = ann
                .and_then(|a| a.resolved_property.as_ref())
                .cloned()
                .unwrap_or_default();

            let target = format!("{}.{}", entity_id, prop_name);

            let to_value = if operator == "+" || operator == "-" {
                // Arithmetic: emit as expression string.
                Json::String(format!("{} {} {}", target, operator, value_expr))
            } else {
                // Direct set: emit typed value.
                let type_name = ann.and_then(|a| a.resolved_type.as_ref());
                typed_value(value_expr, type_name, &prop_name, symbol_table)
            };

            let mut obj = Map::new();
            obj.insert("set".to_string(), Json::String(target));
            obj.insert("to".to_string(), to_value);
            Json::Object(obj)
        }

        EffectType::Move {
            entity_ref: _,
            destination_ref: _,
        } => {
            let ann = annotation.as_ref();
            let entity_id = ann
                .and_then(|a| a.resolved_entity.as_ref())
                .map(|e| strip_at(e))
                .unwrap_or_default();

            let destination = if let Some(a) = ann {
                if let Some(dk) = &a.destination_kind {
                    match dk {
                        DestinationKind::KeywordPlayer => "player".to_string(),
                        DestinationKind::KeywordHere => "player.container".to_string(),
                        DestinationKind::EntityRef(id) => strip_at(id),
                        DestinationKind::LocationRef(id) => id.clone(),
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let mut obj = Map::new();
            obj.insert("move".to_string(), Json::String(entity_id));
            obj.insert("to".to_string(), Json::String(destination));
            Json::Object(obj)
        }

        EffectType::Reveal { target_prop: _ } => {
            let ann = annotation.as_ref();
            let entity_id = ann
                .and_then(|a| a.resolved_entity.as_ref())
                .map(|e| strip_at(e))
                .unwrap_or_default();
            let prop_name = ann
                .and_then(|a| a.resolved_property.as_ref())
                .cloned()
                .unwrap_or_default();

            let mut obj = Map::new();
            obj.insert(
                "reveal".to_string(),
                Json::String(format!("{}.{}", entity_id, prop_name)),
            );
            Json::Object(obj)
        }

        EffectType::Destroy { entity_ref: _ } => {
            let ann = annotation.as_ref();
            let entity_id = ann
                .and_then(|a| a.resolved_entity.as_ref())
                .map(|e| strip_at(e))
                .unwrap_or_default();

            let mut obj = Map::new();
            obj.insert("destroy".to_string(), Json::String(entity_id));
            Json::Object(obj)
        }
    }
}

/// Convert a value expression string to a typed JSON value.
fn typed_value(
    value_expr: &str,
    type_name: Option<&String>,
    prop_name: &str,
    symbol_table: &SymbolTable,
) -> Json {
    // Look up the property type from the symbol table.
    let prop_type = type_name.and_then(|tn| {
        symbol_table
            .types
            .get(tn)
            .and_then(|ts| ts.properties.get(prop_name))
            .map(|ps| &ps.property_type)
    });

    match prop_type {
        Some(PropertyType::Boolean) => match value_expr {
            "true" => Json::Bool(true),
            "false" => Json::Bool(false),
            _ => Json::String(value_expr.to_string()),
        },
        Some(PropertyType::Integer) => {
            if let Ok(i) = value_expr.parse::<i64>() {
                Json::Number(Number::from(i))
            } else {
                Json::String(value_expr.to_string())
            }
        }
        Some(PropertyType::Number) => {
            if let Ok(n) = value_expr.parse::<f64>() {
                number_to_json(n)
            } else {
                Json::String(value_expr.to_string())
            }
        }
        _ => {
            // Enum, String, Ref, or unknown: emit as string.
            Json::String(value_expr.to_string())
        }
    }
}

// ── Helpers ──

fn collect_choice_conditions_effects(
    choice: &crate::ast::Choice,
    symbol_table: &SymbolTable,
) -> (Option<Json>, Vec<Json>) {
    let mut and_conditions: Vec<String> = Vec::new();
    let mut or_conditions: Option<Vec<String>> = None;
    let mut effects: Vec<Json> = Vec::new();

    for child in &choice.content {
        match child {
            ContentNode::Condition(cond) => {
                and_conditions.push(lower_condition(&cond.expr, symbol_table));
            }
            ContentNode::OrConditionBlock(or) => {
                let or_conds: Vec<String> = or
                    .conditions
                    .iter()
                    .map(|c| lower_condition(c, symbol_table))
                    .collect();
                or_conditions = Some(or_conds);
            }
            ContentNode::Effect(eff) => {
                effects.push(lower_effect(&eff.effect_type, &eff.annotation, symbol_table));
            }
            _ => {}
        }
    }

    let conditions = build_conditions_json(&and_conditions, &or_conditions);
    (conditions, effects)
}

fn build_conditions_json(
    and_conditions: &[String],
    or_conditions: &Option<Vec<String>>,
) -> Option<Json> {
    if let Some(or_conds) = or_conditions {
        if !or_conds.is_empty() {
            let mut obj = Map::new();
            obj.insert(
                "any".to_string(),
                Json::Array(or_conds.iter().map(|c| Json::String(c.clone())).collect()),
            );
            return Some(Json::Object(obj));
        }
    }
    if !and_conditions.is_empty() {
        return Some(Json::Array(
            and_conditions.iter().map(|c| Json::String(c.clone())).collect(),
        ));
    }
    None
}

fn value_to_json(value: &Value) -> Json {
    match value {
        Value::String(s) => Json::String(s.clone()),
        Value::Integer(i) => Json::Number(Number::from(*i)),
        Value::Number(n) => number_to_json(*n),
        Value::Boolean(b) => Json::Bool(*b),
        Value::EntityRef(s) => Json::String(strip_at(s)),
        Value::List(items) => Json::Array(items.iter().map(value_to_json).collect()),
    }
}

fn number_to_json(n: f64) -> Json {
    // If the number is a whole number, emit as integer.
    if n.fract() == 0.0 && n.is_finite() {
        if let Some(num) = Number::from_f64(n) {
            return Json::Number(num);
        }
    }
    // Otherwise emit as float.
    Number::from_f64(n)
        .map(Json::Number)
        .unwrap_or(Json::Null)
}

fn scalar_to_json(scalar: &Scalar) -> Json {
    match scalar {
        Scalar::String(s) => Json::String(s.clone()),
        Scalar::Integer(i) => Json::Number(Number::from(*i)),
        Scalar::Number(n) => number_to_json(*n),
        Scalar::Boolean(b) => Json::Bool(*b),
        Scalar::List(items) => Json::Array(items.iter().map(scalar_to_json).collect()),
        Scalar::EntityRef(id) => Json::String(id.clone()),
    }
}

fn format_property_type(pt: &PropertyType) -> &'static str {
    match pt {
        PropertyType::Boolean => "boolean",
        PropertyType::Integer => "integer",
        PropertyType::Number => "number",
        PropertyType::String => "string",
        PropertyType::Enum => "enum",
        PropertyType::Ref => "ref",
        PropertyType::List => "list",
    }
}

fn strip_at(s: &str) -> String {
    s.strip_prefix('@').unwrap_or(s).to_string()
}
