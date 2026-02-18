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

use crate::ast::{ContentNode, FrontmatterValue};
use crate::diagnostics::DiagnosticCollector;
use crate::graph::{DependencyGraph, WARN_CHOICE_NESTING_DEPTH, MAX_CHOICE_NESTING_DEPTH};
use crate::symbol_table::SymbolTable;

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
