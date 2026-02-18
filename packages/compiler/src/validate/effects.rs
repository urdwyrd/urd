/// Step 5: Effect Validation
///
/// Walk all files, for each Effect node:
/// - Set: value type match (URD401), arithmetic operator check (URD424)
/// - Move: portable trait (URD425), destination kind check (URD422)
/// - Reveal: visibility check (URD426 warning)
/// - Destroy: no additional checks

use crate::ast::{ContentNode, DestinationKind, EffectType};
use crate::diagnostics::DiagnosticCollector;
use crate::graph::DependencyGraph;
use crate::symbol_table::{PropertyType, SymbolTable, Visibility};

use super::helpers::{format_property_type, has_trait, parse_string_to_value};

/// Validate all effects in the compilation unit.
pub fn validate_effects(
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

        for content in &node.ast.content {
            validate_content_effects(content, symbol_table, diagnostics);
        }
    }
}

fn validate_content_effects(
    node: &ContentNode,
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    match node {
        ContentNode::Effect(effect) => {
            validate_effect(&effect.effect_type, &effect.annotation, &effect.span, symbol_table, diagnostics);
        }
        ContentNode::Choice(choice) => {
            for child in &choice.content {
                validate_content_effects(child, symbol_table, diagnostics);
            }
        }
        ContentNode::ExitDeclaration(exit) => {
            for child in &exit.children {
                validate_content_effects(child, symbol_table, diagnostics);
            }
        }
        ContentNode::RuleBlock(rule) => {
            for effect in &rule.effects {
                validate_effect(&effect.effect_type, &effect.annotation, &effect.span, symbol_table, diagnostics);
            }
        }
        _ => {}
    }
}

fn validate_effect(
    effect_type: &EffectType,
    annotation: &Option<crate::ast::Annotation>,
    span: &crate::span::Span,
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    match effect_type {
        EffectType::Set { target_prop: _, operator, value_expr } => {
            // Skip if annotation unresolved.
            let ann = match annotation {
                Some(a) => a,
                None => return,
            };
            let entity_id = match &ann.resolved_entity {
                Some(id) => id,
                None => return,
            };
            let prop_name = match &ann.resolved_property {
                Some(p) => p,
                None => return,
            };
            let type_name = match &ann.resolved_type {
                Some(t) => t,
                None => return,
            };

            let type_sym = match symbol_table.types.get(type_name) {
                Some(t) => t,
                None => return,
            };
            let prop = match type_sym.properties.get(prop_name) {
                Some(p) => p,
                None => return,
            };

            // Check arithmetic operators.
            if operator == "+" || operator == "-" {
                if !matches!(prop.property_type, PropertyType::Integer | PropertyType::Number) {
                    diagnostics.error(
                        "URD424",
                        format!(
                            "Arithmetic operator '{}' is not valid for property '{}' of type '{}'. Arithmetic effects require integer or number properties.",
                            operator, prop.name, format_property_type(&prop.property_type),
                        ),
                        span.clone(),
                    );
                    return;
                }

                // Check that value is numeric.
                let value = parse_string_to_value(value_expr, &prop.property_type);
                match &value {
                    crate::symbol_table::Value::Integer(_) | crate::symbol_table::Value::Number(_) => {}
                    _ => {
                        diagnostics.error(
                            "URD401",
                            format!(
                                "Type mismatch: property '{}' on entity '@{}' expects {} but got '{}'.",
                                prop.name, entity_id, format_property_type(&prop.property_type), value_expr,
                            ),
                            span.clone(),
                        );
                    }
                }
            } else {
                // Regular set: type-check the value.
                let value = parse_string_to_value(value_expr, &prop.property_type);
                super::helpers::check_value(
                    &value,
                    prop,
                    entity_id,
                    type_name,
                    super::helpers::CheckContext::ConditionOrEffect,
                    symbol_table,
                    span,
                    diagnostics,
                );
            }
        }

        EffectType::Move { entity_ref: _, destination_ref: _ } => {
            // Skip if annotation unresolved.
            let ann = match annotation {
                Some(a) => a,
                None => return,
            };
            let entity_id = match &ann.resolved_entity {
                Some(id) => id,
                None => return,
            };

            // Check portable trait on moved entity.
            if let Some(es) = symbol_table.entities.get(entity_id) {
                if let Some(type_name) = &es.type_symbol {
                    if !has_trait(type_name, "portable", symbol_table) {
                        diagnostics.error(
                            "URD425",
                            format!(
                                "Entity '@{}' cannot be moved because its type '{}' does not have the 'portable' trait.",
                                entity_id, type_name,
                            ),
                            span.clone(),
                        );
                    }
                }
                // If type_symbol is None, skip — LINK already reported URD307.
            }

            // Check destination via destination_kind discriminator.
            match &ann.destination_kind {
                Some(DestinationKind::KeywordPlayer) | Some(DestinationKind::KeywordHere) => {
                    // Always valid.
                }
                Some(DestinationKind::LocationRef(_)) => {
                    // Locations are containers by definition.
                }
                Some(DestinationKind::EntityRef(dest_id)) => {
                    if let Some(dest_es) = symbol_table.entities.get(dest_id) {
                        if let Some(dest_type) = &dest_es.type_symbol {
                            if !has_trait(dest_type, "container", symbol_table) {
                                diagnostics.error(
                                    "URD422",
                                    format!(
                                        "Entity '@{}' is used as a container in move destination but its type '{}' does not have the 'container' trait.",
                                        dest_id, dest_type,
                                    ),
                                    span.clone(),
                                );
                            }
                        }
                        // If type_symbol is None, skip — no cascading.
                    }
                }
                None => {
                    // Destination unresolved — LINK already emitted diagnostic.
                    return;
                }
            }
        }

        EffectType::Reveal { target_prop: _ } => {
            // Skip if annotation unresolved.
            let ann = match annotation {
                Some(a) => a,
                None => return,
            };
            let entity_id = match &ann.resolved_entity {
                Some(id) => id,
                None => return,
            };
            let prop_name = match &ann.resolved_property {
                Some(p) => p,
                None => return,
            };
            let type_name = match &ann.resolved_type {
                Some(t) => t,
                None => return,
            };

            let type_sym = match symbol_table.types.get(type_name) {
                Some(t) => t,
                None => return,
            };
            let prop = match type_sym.properties.get(prop_name) {
                Some(p) => p,
                None => return,
            };

            // Check that property is hidden.
            if prop.visibility != Visibility::Hidden {
                diagnostics.warning(
                    "URD426",
                    format!(
                        "Property '{}' on entity '@{}' is not hidden. Reveal has no effect.",
                        prop.name, entity_id,
                    ),
                    span.clone(),
                );
            }
        }

        EffectType::Destroy { entity_ref: _ } => {
            // No additional type checks needed. Any entity can be destroyed.
        }
    }
}
