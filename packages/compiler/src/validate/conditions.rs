/// Step 4: Condition Validation
///
/// Walk all files in topological order, recurse into content nodes:
/// - PropertyComparison: operator compatibility (URD420), value type match (URD401)
/// - ContainmentCheck: container trait check via container_kind (URD422)
/// - ExhaustionCheck: file-locality check (URD423)

use crate::ast::{ConditionExpr, ContainerKind, ContentNode};
use crate::diagnostics::DiagnosticCollector;
use crate::graph::DependencyGraph;
use crate::span::Span;
use crate::symbol_table::{PropertyType, SymbolTable};

use super::helpers::{format_property_type, has_trait, parse_string_to_value};

/// Validate all conditions in the compilation unit.
pub fn validate_conditions(
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

        // Collect local section IDs for exhaustion checks.
        let file_stem = crate::graph::file_stem(file_path);
        let local_section_ids: Vec<String> = symbol_table
            .sections
            .values()
            .filter(|s| s.file_stem == file_stem)
            .map(|s| s.local_name.clone())
            .collect();

        for content in &node.ast.content {
            validate_content_conditions(content, file_path, &local_section_ids, symbol_table, diagnostics);
        }
    }
}

fn validate_content_conditions(
    node: &ContentNode,
    file_path: &str,
    local_section_ids: &[String],
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    match node {
        ContentNode::Condition(cond) => {
            validate_condition_expr(&cond.expr, file_path, local_section_ids, symbol_table, diagnostics);
        }
        ContentNode::OrConditionBlock(or_block) => {
            for expr in &or_block.conditions {
                validate_condition_expr(expr, file_path, local_section_ids, symbol_table, diagnostics);
            }
        }
        ContentNode::Choice(choice) => {
            for child in &choice.content {
                validate_content_conditions(child, file_path, local_section_ids, symbol_table, diagnostics);
            }
        }
        ContentNode::ExitDeclaration(exit) => {
            for child in &exit.children {
                validate_content_conditions(child, file_path, local_section_ids, symbol_table, diagnostics);
            }
        }
        ContentNode::RuleBlock(rule) => {
            for expr in &rule.where_clauses {
                validate_condition_expr(expr, file_path, local_section_ids, symbol_table, diagnostics);
            }
            if let Some(select) = &rule.select {
                for expr in &select.where_clauses {
                    validate_condition_expr(expr, file_path, local_section_ids, symbol_table, diagnostics);
                }
            }
        }
        _ => {}
    }
}

fn validate_condition_expr(
    expr: &ConditionExpr,
    _file_path: &str,
    local_section_ids: &[String],
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    match expr {
        ConditionExpr::PropertyComparison(pc) => {
            // Skip if entity/property unresolved.
            let ann = match &pc.annotation {
                Some(a) => a,
                None => return,
            };
            if ann.resolved_entity.is_none() || ann.resolved_property.is_none() {
                return;
            }
            let entity_id = ann.resolved_entity.as_ref().unwrap();
            let resolved_type = match &ann.resolved_type {
                Some(t) => t,
                None => return,
            };
            let prop_name = ann.resolved_property.as_ref().unwrap();

            let type_sym = match symbol_table.types.get(resolved_type) {
                Some(t) => t,
                None => return,
            };
            let prop = match type_sym.properties.get(prop_name) {
                Some(p) => p,
                None => return,
            };

            // 1. Operator compatibility.
            validate_operator(&pc.operator, prop, &pc.span, diagnostics);

            // 2. Value type check.
            let value = parse_string_to_value(&pc.value, &prop.property_type);
            validate_comparison_value(&value, prop, entity_id, &pc.span, diagnostics);
        }

        ConditionExpr::ContainmentCheck(cc) => {
            // Skip if entity unresolved.
            let ann = match &cc.annotation {
                Some(a) => a,
                None => return,
            };
            if ann.resolved_entity.is_none() {
                return;
            }

            // Check container via container_kind discriminator.
            match &ann.container_kind {
                Some(ContainerKind::KeywordPlayer) | Some(ContainerKind::KeywordHere) => {
                    // Always valid. No trait checks.
                }
                Some(ContainerKind::LocationRef(_)) => {
                    // Locations are containers by definition.
                }
                Some(ContainerKind::EntityRef(container_id)) => {
                    // Check container trait on entity's type.
                    if let Some(es) = symbol_table.entities.get(container_id) {
                        if let Some(type_name) = &es.type_symbol {
                            if !has_trait(type_name, "container", symbol_table) {
                                diagnostics.error(
                                    "URD422",
                                    format!(
                                        "Entity '@{}' is used as a container in containment check but its type '{}' does not have the 'container' trait.",
                                        container_id, type_name,
                                    ),
                                    cc.span.clone(),
                                );
                            }
                        }
                        // If type_symbol is None, skip — LINK already reported URD307.
                    }
                }
                None => {
                    // Container unresolved — LINK already emitted diagnostic. Skip.
                    return;
                }
            }
        }

        ConditionExpr::ExhaustionCheck(ec) => {
            // Skip if section unresolved (annotation is None).
            let ann = match &ec.annotation {
                Some(a) => a,
                None => return,
            };
            // The section must be file-local. LINK resolves section names to compiled IDs
            // from the current file's local_sections. If it resolved, it's file-local.
            // If the annotation has a resolved_section, LINK already confirmed file-locality.
            // VALIDATE confirms by checking the section's local_name is in this file's sections.
            if ann.resolved_section.is_none() {
                return;
            }
            // Check file-locality: the section_name must be in local_section_ids.
            if !local_section_ids.contains(&ec.section_name) {
                diagnostics.error(
                    "URD423",
                    format!(
                        "Exhaustion check references section '{}' which is not declared in this file.",
                        ec.section_name,
                    ),
                    ec.span.clone(),
                );
            }
        }
    }
}

fn validate_operator(
    operator: &str,
    prop: &crate::symbol_table::PropertySymbol,
    span: &Span,
    diagnostics: &mut DiagnosticCollector,
) {
    // == and != are valid for all types.
    if operator == "==" || operator == "!=" {
        return;
    }
    // <, >, <=, >= are valid only for integer and number.
    if matches!(operator, "<" | ">" | "<=" | ">=") {
        if !matches!(prop.property_type, PropertyType::Integer | PropertyType::Number) {
            diagnostics.error(
                "URD420",
                format!(
                    "Operator '{}' is not valid for property '{}' of type '{}'. Use == or != for non-numeric types.",
                    operator, prop.name, format_property_type(&prop.property_type),
                ),
                span.clone(),
            );
        }
    }
}

fn validate_comparison_value(
    value: &crate::symbol_table::Value,
    prop: &crate::symbol_table::PropertySymbol,
    entity_id: &str,
    span: &Span,
    diagnostics: &mut DiagnosticCollector,
) {
    use crate::symbol_table::Value;
    match &prop.property_type {
        PropertyType::Boolean => {
            if !matches!(value, Value::Boolean(_)) {
                diagnostics.error(
                    "URD401",
                    format!(
                        "Type mismatch: property '{}' on entity '@{}' expects boolean but got '{}'.",
                        prop.name, entity_id, super::helpers::format_value(value),
                    ),
                    span.clone(),
                );
            }
        }
        PropertyType::Integer => {
            if !matches!(value, Value::Integer(_)) {
                diagnostics.error(
                    "URD401",
                    format!(
                        "Type mismatch: property '{}' on entity '@{}' expects integer but got '{}'.",
                        prop.name, entity_id, super::helpers::format_value(value),
                    ),
                    span.clone(),
                );
            }
        }
        PropertyType::Number => {
            if !matches!(value, Value::Number(_) | Value::Integer(_)) {
                diagnostics.error(
                    "URD401",
                    format!(
                        "Type mismatch: property '{}' on entity '@{}' expects number but got '{}'.",
                        prop.name, entity_id, super::helpers::format_value(value),
                    ),
                    span.clone(),
                );
            }
        }
        PropertyType::Enum => {
            if let Value::String(s) = value {
                if let Some(values) = &prop.values {
                    if !values.contains(s) {
                        diagnostics.error(
                            "URD401",
                            format!(
                                "Type mismatch: property '{}' on entity '@{}' expects enum but got '{}'.",
                                prop.name, entity_id, s,
                            ),
                            span.clone(),
                        );
                    }
                }
            }
        }
        PropertyType::String => {
            // String comparisons always match.
        }
        _ => {}
    }
}
