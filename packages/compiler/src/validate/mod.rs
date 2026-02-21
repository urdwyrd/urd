/// Phase 4: VALIDATE — annotated ASTs + symbol table to diagnostics.
///
/// Input:  `LinkedWorld` (annotated ASTs + `SymbolTable`)
/// Output: Validation diagnostics (appended to collector)
///
/// VALIDATE is read-only — it checks everything, modifies nothing.
///
/// The skip rule: if an annotation is `null`, VALIDATE silently skips
/// every check that depends on it. LINK already reported the root cause.
///
/// Key guarantee: all type constraints checked, unresolved references
/// silently skipped (no cascading errors).
///
/// Diagnostic code range: URD400–URD499

mod helpers;
mod types;
mod entities;
mod conditions;
mod effects;

use std::collections::{HashSet, VecDeque};

use crate::ast::{Choice, ConditionExpr, ContentNode, FrontmatterValue, PropertyComparison};
use crate::diagnostics::DiagnosticCollector;
use crate::graph::{DependencyGraph, WARN_CHOICE_NESTING_DEPTH, MAX_CHOICE_NESTING_DEPTH};
use crate::slugify::slugify;
use crate::symbol_table::{PropertyType, SymbolTable};

/// Valid advance modes for sequence phases.
const VALID_ADVANCE_MODES: &[&str] = &["on_action", "on_rule", "on_condition", "end", "auto", "manual"];

/// Validate the linked world: type-check properties, conditions, effects.
/// Enforce all semantic constraints defined in the spec.
pub fn validate(
    graph: &DependencyGraph,
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    let ordered: Vec<String> = graph.topological_order().into_iter().cloned().collect();

    // Step 1: Global configuration checks.
    validate_global_config(graph, &ordered, symbol_table, diagnostics);

    // Step 2: Type definition validation.
    types::validate_types(symbol_table, diagnostics);

    // Step 3: Entity property override validation.
    entities::validate_entities(symbol_table, diagnostics);

    // Step 4: Condition validation.
    conditions::validate_conditions(graph, &ordered, symbol_table, diagnostics);

    // Step 5: Effect validation.
    effects::validate_effects(graph, &ordered, symbol_table, diagnostics);

    // Step 6: Action validation.
    validate_actions(symbol_table, diagnostics);

    // Step 7: Sequence and phase validation.
    validate_sequences(symbol_table, diagnostics);

    // Step 8: Nesting depth validation.
    validate_nesting_depth(graph, &ordered, diagnostics);

    // Step 9: Unreachable location (S3).
    validate_location_reachability(symbol_table, diagnostics);

    // Step 10: Orphaned choice (S4).
    validate_orphaned_choices(graph, &ordered, symbol_table, diagnostics);

    // Step 11: Missing fallthrough (S6).
    validate_section_fallthrough(graph, &ordered, symbol_table, diagnostics);

    // Step 12: Section-exit shadowing (S8).
    validate_section_exit_shadowing(graph, &ordered, symbol_table, diagnostics);
}

// ── Step 1: Global Configuration ──

fn validate_global_config(
    graph: &DependencyGraph,
    ordered_asts: &[String],
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    // Find the world block in any file's frontmatter.
    let mut world_start_raw: Option<String> = None;
    let mut world_entry_raw: Option<String> = None;
    let mut world_span = None;
    let mut has_urd_field = false;

    for file_path in ordered_asts {
        let node = match graph.nodes.get(file_path.as_str()) {
            Some(n) => n,
            None => continue,
        };
        if let Some(fm) = &node.ast.frontmatter {
            for entry in &fm.entries {
                if let FrontmatterValue::WorldBlock(wb) = &entry.value {
                    world_span = Some(wb.span.clone());
                    for (key, scalar) in &wb.fields {
                        match key.as_str() {
                            "start" => {
                                if let crate::ast::Scalar::String(s) = scalar {
                                    world_start_raw = Some(s.clone());
                                }
                            }
                            "entry" => {
                                if let crate::ast::Scalar::String(s) = scalar {
                                    world_entry_raw = Some(s.clone());
                                }
                            }
                            "urd" => {
                                has_urd_field = true;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // a. world.start
    if let Some(start_val) = &world_start_raw {
        if symbol_table.world_start.is_none() {
            if let Some(ref ws) = world_span {
                diagnostics.error(
                    "URD404",
                    format!(
                        "world.start references '{}' but no location with that ID exists.",
                        start_val,
                    ),
                    ws.clone(),
                );
            }
        }
    }

    // b. world.entry
    if let Some(entry_val) = &world_entry_raw {
        if symbol_table.world_entry.is_none() {
            if let Some(ref ws) = world_span {
                diagnostics.error(
                    "URD405",
                    format!(
                        "world.entry references '{}' but no sequence with that ID exists.",
                        entry_val,
                    ),
                    ws.clone(),
                );
            }
        }
    }

    // c. urd field override
    if has_urd_field {
        if let Some(ref ws) = world_span {
            diagnostics.warning(
                "URD411",
                "The 'urd' field is set automatically by the compiler. Author value will be overridden.",
                ws.clone(),
            );
        }
    }

    // d. Player entity traits
    if let Some(player) = symbol_table.entities.get("player") {
        if let Some(type_name) = &player.type_symbol {
            if let Some(type_sym) = symbol_table.types.get(type_name) {
                let has_mobile = type_sym.traits.iter().any(|t| t == "mobile");
                let has_container = type_sym.traits.iter().any(|t| t == "container");
                if !has_mobile {
                    diagnostics.error(
                        "URD412",
                        format!(
                            "Player entity '@player' has type '{}' which is missing required trait 'mobile'. The player type must have both 'mobile' and 'container' traits.",
                            type_name,
                        ),
                        player.declared_in.clone(),
                    );
                }
                if !has_container {
                    diagnostics.error(
                        "URD412",
                        format!(
                            "Player entity '@player' has type '{}' which is missing required trait 'container'. The player type must have both 'mobile' and 'container' traits.",
                            type_name,
                        ),
                        player.declared_in.clone(),
                    );
                }
            }
        }
    }
}

// ── Step 6: Action Validation ──

fn validate_actions(
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    for (action_id, action_sym) in &symbol_table.actions {
        // Mutual exclusion: both target + target_type.
        if action_sym.target.is_some() && action_sym.target_type.is_some() {
            diagnostics.error(
                "URD406",
                format!(
                    "Action '{}' declares both 'target' and 'target_type'. Declare one or neither.",
                    action_id,
                ),
                action_sym.declared_in.clone(),
            );
        }
    }
}

// ── Step 7: Sequence and Phase Validation ──

fn validate_sequences(
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    for (sequence_id, sequence_sym) in &symbol_table.sequences {
        // Empty sequence.
        if sequence_sym.phases.is_empty() {
            diagnostics.error(
                "URD428",
                format!(
                    "Sequence '{}' declares no phases.",
                    sequence_id,
                ),
                sequence_sym.declared_in.clone(),
            );
        }

        for phase in &sequence_sym.phases {
            // a. Phase action references.
            if let Some(action_ref) = &phase.action {
                if !symbol_table.actions.contains_key(action_ref) {
                    diagnostics.error(
                        "URD407",
                        format!(
                            "Phase '{}' in sequence '{}' references unknown action '{}'.",
                            phase.id, sequence_id, action_ref,
                        ),
                        phase.declared_in.clone(),
                    );
                }
            }
            if let Some(actions) = &phase.actions {
                for action_ref in actions {
                    if !symbol_table.actions.contains_key(action_ref) {
                        diagnostics.error(
                            "URD407",
                            format!(
                                "Phase '{}' in sequence '{}' references unknown action '{}'.",
                                phase.id, sequence_id, action_ref,
                            ),
                            phase.declared_in.clone(),
                        );
                    }
                }
            }

            // b. Phase rule references.
            if let Some(rule_ref) = &phase.rule {
                if !symbol_table.rules.contains_key(rule_ref) {
                    diagnostics.error(
                        "URD408",
                        format!(
                            "Phase '{}' in sequence '{}' references unknown rule '{}'.",
                            phase.id, sequence_id, rule_ref,
                        ),
                        phase.declared_in.clone(),
                    );
                }
            }

            // c. Advance mode validation.
            if !VALID_ADVANCE_MODES.contains(&phase.advance.as_str()) {
                diagnostics.error(
                    "URD409",
                    format!(
                        "Invalid advance mode '{}' in phase '{}'. Valid modes: on_action, on_rule, on_condition, end.",
                        phase.advance, phase.id,
                    ),
                    phase.declared_in.clone(),
                );
            }

            // d. Auto phase with player actions.
            // Check if advance is "auto" and phase has actions.
            if phase.advance == "auto" {
                let has_actions = phase.action.is_some()
                    || phase.actions.as_ref().map(|a| !a.is_empty()).unwrap_or(false);
                if has_actions {
                    diagnostics.warning(
                        "URD427",
                        format!(
                            "Phase '{}' is auto-advancing but declares player actions. The actions will not be available.",
                            phase.id,
                        ),
                        phase.declared_in.clone(),
                    );
                }
            }
        }
    }
}

// ── Step 8: Nesting Depth Validation ──

fn validate_nesting_depth(
    graph: &DependencyGraph,
    ordered_asts: &[String],
    diagnostics: &mut DiagnosticCollector,
) {
    for file_path in ordered_asts {
        let node = match graph.nodes.get(file_path.as_str()) {
            Some(n) => n,
            None => continue,
        };

        for content in &node.ast.content {
            check_nesting(content, diagnostics);
        }
    }
}

fn check_nesting(
    node: &ContentNode,
    diagnostics: &mut DiagnosticCollector,
) {
    match node {
        ContentNode::Choice(choice) => {
            let depth = choice.indent_level;
            if depth >= MAX_CHOICE_NESTING_DEPTH {
                diagnostics.error(
                    "URD410",
                    format!("Nesting depth {} at line {}.", depth, choice.span.start_line),
                    choice.span.clone(),
                );
            } else if depth >= WARN_CHOICE_NESTING_DEPTH {
                diagnostics.warning(
                    "URD410",
                    format!("Nesting depth {} at line {}.", depth, choice.span.start_line),
                    choice.span.clone(),
                );
            }

            for child in &choice.content {
                check_nesting(child, diagnostics);
            }
        }
        ContentNode::ExitDeclaration(exit) => {
            for child in &exit.children {
                check_nesting(child, diagnostics);
            }
        }
        _ => {}
    }
}

// ── Step 9: Unreachable Location (S3) ──

fn validate_location_reachability(
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    // Skip if no world.start defined — unreachability is meaningless without a root.
    let start_id = match &symbol_table.world_start {
        Some(id) => id,
        None => return,
    };

    // Skip if start location not in symbol table (URD404 already covers this).
    if !symbol_table.locations.contains_key(start_id) {
        return;
    }

    // BFS from start.
    let mut visited: HashSet<&str> = HashSet::new();
    let mut queue: VecDeque<&str> = VecDeque::new();
    queue.push_back(start_id.as_str());
    visited.insert(start_id.as_str());

    while let Some(current) = queue.pop_front() {
        if let Some(loc_sym) = symbol_table.locations.get(current) {
            for exit in loc_sym.exits.values() {
                if let Some(dest) = &exit.resolved_destination {
                    if visited.insert(dest.as_str()) {
                        queue.push_back(dest.as_str());
                    }
                }
            }
        }
    }

    // Report unreachable locations in insertion order.
    for (loc_id, loc_sym) in &symbol_table.locations {
        if !visited.contains(loc_id.as_str()) {
            diagnostics.warning(
                "URD430",
                format!(
                    "Location '{}' is unreachable. No path from the start location reaches it.",
                    loc_id,
                ),
                loc_sym.declared_in.clone(),
            );
        }
    }
}

// ── Step 10: Orphaned Choice (S4) ──

fn validate_orphaned_choices(
    graph: &DependencyGraph,
    ordered_asts: &[String],
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    for file_path in ordered_asts {
        let node = match graph.nodes.get(file_path.as_str()) {
            Some(n) => n,
            None => continue,
        };

        let mut current_section: Option<String> = None;

        for content in &node.ast.content {
            match content {
                ContentNode::SectionLabel(sl) => {
                    current_section = Some(sl.name.clone());
                }
                ContentNode::Choice(choice) => {
                    check_choice_orphaned(
                        choice,
                        current_section.as_deref().unwrap_or("unnamed"),
                        file_path,
                        symbol_table,
                        diagnostics,
                    );
                }
                _ => {}
            }
        }
    }
}

fn check_choice_orphaned(
    choice: &Choice,
    section_name: &str,
    file_path: &str,
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    for child in &choice.content {
        match child {
            ContentNode::Condition(cond) => {
                if let ConditionExpr::PropertyComparison(pc) = &cond.expr {
                    check_enum_condition(pc, choice, section_name, file_path, symbol_table, diagnostics);
                }
            }
            ContentNode::OrConditionBlock(or_block) => {
                for expr in &or_block.conditions {
                    if let ConditionExpr::PropertyComparison(pc) = expr {
                        check_enum_condition(pc, choice, section_name, file_path, symbol_table, diagnostics);
                    }
                }
            }
            ContentNode::Choice(nested) => {
                check_choice_orphaned(nested, section_name, file_path, symbol_table, diagnostics);
            }
            _ => {}
        }
    }
}

fn check_enum_condition(
    pc: &PropertyComparison,
    choice: &Choice,
    section_name: &str,
    file_path: &str,
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    // Skip rule: annotation must be fully resolved.
    let ann = match &pc.annotation {
        Some(a) => a,
        None => return,
    };
    if ann.resolved_entity.is_none() || ann.resolved_type.is_none() || ann.resolved_property.is_none() {
        return;
    }

    let resolved_type = ann.resolved_type.as_ref().unwrap();
    let resolved_property = ann.resolved_property.as_ref().unwrap();

    // Only check == operator on enum properties.
    if pc.operator != "==" {
        return;
    }

    let type_sym = match symbol_table.types.get(resolved_type) {
        Some(t) => t,
        None => return,
    };
    let prop = match type_sym.properties.get(resolved_property) {
        Some(p) => p,
        None => return,
    };

    if prop.property_type != PropertyType::Enum {
        return;
    }

    if let Some(values) = &prop.values {
        if !values.contains(&pc.value) {
            diagnostics.warning(
                "URD432",
                format!(
                    "Choice in section '{}' (file '{}') may never be available. Condition requires '{}' == '{}' but type '{}' only allows: [{}].",
                    section_name,
                    file_path,
                    resolved_property,
                    pc.value,
                    resolved_type,
                    values.join(", "),
                ),
                choice.span.clone(),
            );
        }
    }
}

// ── Step 11: Missing Fallthrough (S6) ──

fn validate_section_fallthrough(
    graph: &DependencyGraph,
    ordered_asts: &[String],
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    for file_path in ordered_asts {
        let node = match graph.nodes.get(file_path.as_str()) {
            Some(n) => n,
            None => continue,
        };

        let file_stem = crate::graph::file_stem(file_path);
        let content = &node.ast.content;

        // Collect section start positions: (name, span, index).
        let section_starts: Vec<(String, crate::span::Span, usize)> = content
            .iter()
            .enumerate()
            .filter_map(|(i, c)| {
                if let ContentNode::SectionLabel(sl) = c {
                    Some((sl.name.clone(), sl.span.clone(), i))
                } else {
                    None
                }
            })
            .collect();

        for (idx, (section_name, section_span, start)) in section_starts.iter().enumerate() {
            let end = if idx + 1 < section_starts.len() {
                section_starts[idx + 1].2
            } else {
                content.len()
            };

            let section_content = &content[*start..end];

            // Look up section in symbol table.
            let compiled_id = format!("{}/{}", file_stem, section_name);
            let section_sym = match symbol_table.sections.get(&compiled_id) {
                Some(s) => s,
                None => continue,
            };

            // Condition 4: No choices at all — skip.
            if section_sym.choices.is_empty() {
                continue;
            }

            // Condition 1: Has at least one sticky choice — safe.
            if section_sym.choices.iter().any(|c| c.sticky) {
                continue;
            }

            // Find last Choice node in section content.
            let last_choice_idx = match section_content.iter().rposition(|c| matches!(c, ContentNode::Choice(_))) {
                Some(idx) => idx,
                None => continue,
            };

            // Conditions 2 and 3: check content after the last choice for fallthrough.
            let has_fallthrough = section_content[last_choice_idx + 1..].iter().any(|c| {
                matches!(
                    c,
                    ContentNode::Jump(_)
                        | ContentNode::Prose(_)
                        | ContentNode::EntitySpeech(_)
                        | ContentNode::StageDirection(_)
                )
            });

            if !has_fallthrough {
                diagnostics.warning(
                    "URD433",
                    format!(
                        "Section '{}' in file '{}' has only one-shot choices and no terminal jump or fallthrough text. It will exhaust to an empty state.",
                        section_name, file_path,
                    ),
                    section_span.clone(),
                );
            }
        }
    }
}

// ── Step 12: Section-Exit Shadowing (S8) ──

fn validate_section_exit_shadowing(
    graph: &DependencyGraph,
    ordered_asts: &[String],
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    for file_path in ordered_asts {
        let node = match graph.nodes.get(file_path.as_str()) {
            Some(n) => n,
            None => continue,
        };

        let mut current_location_id: Option<String> = None;

        for content in &node.ast.content {
            match content {
                ContentNode::LocationHeading(lh) => {
                    let slug = slugify(&lh.display_name);
                    if symbol_table.locations.contains_key(&slug) {
                        current_location_id = Some(slug);
                    } else {
                        current_location_id = None;
                    }
                }
                ContentNode::SectionLabel(sl) => {
                    if let Some(ref loc_id) = current_location_id {
                        if let Some(loc_sym) = symbol_table.locations.get(loc_id) {
                            if loc_sym.exits.contains_key(&sl.name) {
                                diagnostics.warning(
                                    "URD434",
                                    format!(
                                        "Section '{}' in location '{}' shares a name with exit '{}'. Jumps to '{}' will target the section, not the exit. Use -> exit:{} to target the exit explicitly.",
                                        sl.name, loc_id, sl.name, sl.name, sl.name,
                                    ),
                                    sl.span.clone(),
                                );
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
