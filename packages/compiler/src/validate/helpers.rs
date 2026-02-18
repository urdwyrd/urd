/// Shared validation helpers used across VALIDATE sub-modules.
///
/// Core responsibilities:
/// - Type-check values against property definitions
/// - Parse raw strings into typed values
/// - Look up traits on types
/// - Look up entity → type symbol chain

use crate::diagnostics::DiagnosticCollector;
use crate::span::Span;
use crate::symbol_table::{PropertySymbol, PropertyType, SymbolTable, Value, Visibility};

/// Check if a type has a given trait.
pub fn has_trait(type_name: &str, trait_name: &str, symbol_table: &SymbolTable) -> bool {
    symbol_table
        .types
        .get(type_name)
        .map(|ts| ts.traits.iter().any(|t| t == trait_name))
        .unwrap_or(false)
}

/// Parse a raw string value into a `Value` given the expected property type.
/// Used for PropertyComparison.value and Set.value_expr.
pub fn parse_string_to_value(raw: &str, expected_type: &PropertyType) -> Value {
    match expected_type {
        PropertyType::Boolean => {
            match raw {
                "true" => Value::Boolean(true),
                "false" => Value::Boolean(false),
                _ => Value::String(raw.to_string()),
            }
        }
        PropertyType::Integer => {
            if let Ok(i) = raw.parse::<i64>() {
                Value::Integer(i)
            } else {
                Value::String(raw.to_string())
            }
        }
        PropertyType::Number => {
            if let Ok(n) = raw.parse::<f64>() {
                Value::Number(n)
            } else {
                Value::String(raw.to_string())
            }
        }
        PropertyType::Enum | PropertyType::String => {
            Value::String(raw.to_string())
        }
        PropertyType::Ref => {
            // Ref values are entity IDs (without @).
            Value::EntityRef(raw.to_string())
        }
        PropertyType::List => {
            // List values are not parsed from raw strings in VALIDATE.
            Value::String(raw.to_string())
        }
    }
}

/// Format a Value for diagnostic messages.
pub fn format_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::EntityRef(s) => format!("@{}", s),
        Value::List(items) => {
            let formatted: Vec<String> = items.iter().map(format_value).collect();
            format!("[{}]", formatted.join(", "))
        }
    }
}

/// Format a PropertyType for diagnostic messages.
pub fn format_property_type(pt: &PropertyType) -> &'static str {
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

/// Context for value type-checking — determines which diagnostic codes to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckContext {
    /// Entity property override (Step 3): uses URD402 for enum mismatches.
    Override,
    /// Type property default (Step 2): uses URD413 for all mismatches.
    Default,
    /// Condition or effect (Steps 4–5): uses URD401 for all mismatches.
    ConditionOrEffect,
}

/// Validate a single value against a property definition.
/// Emits diagnostics as needed. Returns true if the value is valid.
pub fn check_value(
    value: &Value,
    prop: &PropertySymbol,
    entity_id: &str,
    type_name: &str,
    context: CheckContext,
    symbol_table: &SymbolTable,
    span: &Span,
    diagnostics: &mut DiagnosticCollector,
) -> bool {
    let is_default = context == CheckContext::Default;
    match &prop.property_type {
        PropertyType::Boolean => {
            if !matches!(value, Value::Boolean(_)) {
                emit_type_mismatch(value, prop, entity_id, type_name, is_default, span, diagnostics);
                return false;
            }
        }
        PropertyType::Integer => {
            if !matches!(value, Value::Integer(_)) {
                emit_type_mismatch(value, prop, entity_id, type_name, is_default, span, diagnostics);
                return false;
            }
            if let Value::Integer(i) = value {
                return check_range(*i as f64, prop, entity_id, span, diagnostics);
            }
        }
        PropertyType::Number => {
            let num = match value {
                Value::Number(n) => Some(*n),
                Value::Integer(i) => Some(*i as f64),
                _ => None,
            };
            if let Some(n) = num {
                return check_range(n, prop, entity_id, span, diagnostics);
            } else {
                emit_type_mismatch(value, prop, entity_id, type_name, is_default, span, diagnostics);
                return false;
            }
        }
        PropertyType::String => {
            if !matches!(value, Value::String(_)) {
                emit_type_mismatch(value, prop, entity_id, type_name, is_default, span, diagnostics);
                return false;
            }
        }
        PropertyType::Enum => {
            if let Value::String(s) = value {
                if let Some(values) = &prop.values {
                    if !values.contains(s) {
                        match context {
                            CheckContext::Override => {
                                diagnostics.error(
                                    "URD402",
                                    format!(
                                        "Enum value '{}' is not valid for property '{}' on entity '@{}'. Valid values: {}.",
                                        s, prop.name, entity_id,
                                        values.join(", "),
                                    ),
                                    span.clone(),
                                );
                            }
                            CheckContext::Default => {
                                diagnostics.error(
                                    "URD413",
                                    format!(
                                        "Default value '{}' for property '{}' on type '{}' does not match the declared type '{}'.",
                                        s, prop.name, type_name, format_property_type(&prop.property_type),
                                    ),
                                    span.clone(),
                                );
                            }
                            CheckContext::ConditionOrEffect => {
                                diagnostics.error(
                                    "URD401",
                                    format!(
                                        "Type mismatch: property '{}' on entity '@{}' expects {} but got '{}'.",
                                        prop.name, entity_id, format_property_type(&prop.property_type), s,
                                    ),
                                    span.clone(),
                                );
                            }
                        }
                        return false;
                    }
                }
            } else {
                emit_type_mismatch(value, prop, entity_id, type_name, is_default, span, diagnostics);
                return false;
            }
        }
        PropertyType::Ref => {
            if let Value::EntityRef(ref_id) = value {
                check_ref_type(ref_id, prop, entity_id, symbol_table, span, diagnostics);
            } else if let Value::String(ref_id) = value {
                // String values might be entity refs in override context.
                check_ref_type(ref_id, prop, entity_id, symbol_table, span, diagnostics);
            } else {
                emit_type_mismatch(value, prop, entity_id, type_name, is_default, span, diagnostics);
                return false;
            }
        }
        PropertyType::List => {
            if let Value::List(items) = value {
                // Validate each element against element_type.
                if let Some(elem_type) = &prop.element_type {
                    let elem_prop = PropertySymbol {
                        name: prop.name.clone(),
                        property_type: elem_type.clone(),
                        default: None,
                        visibility: Visibility::Visible,
                        values: prop.element_values.clone(),
                        min: None,
                        max: None,
                        ref_type: prop.element_ref_type.clone(),
                        element_type: None,
                        element_values: None,
                        element_ref_type: None,
                        declared_in: prop.declared_in.clone(),
                    };
                    for item in items {
                        check_value(item, &elem_prop, entity_id, type_name, context, symbol_table, span, diagnostics);
                    }
                }
            } else {
                emit_type_mismatch(value, prop, entity_id, type_name, is_default, span, diagnostics);
                return false;
            }
        }
    }
    true
}

fn emit_type_mismatch(
    value: &Value,
    prop: &PropertySymbol,
    entity_id: &str,
    type_name: &str,
    is_default: bool,
    span: &Span,
    diagnostics: &mut DiagnosticCollector,
) {
    if is_default {
        diagnostics.error(
            "URD413",
            format!(
                "Default value '{}' for property '{}' on type '{}' does not match the declared type '{}'.",
                format_value(value), prop.name, type_name, format_property_type(&prop.property_type),
            ),
            span.clone(),
        );
    } else {
        diagnostics.error(
            "URD401",
            format!(
                "Type mismatch: property '{}' on entity '@{}' expects {} but got '{}'.",
                prop.name, entity_id, format_property_type(&prop.property_type), format_value(value),
            ),
            span.clone(),
        );
    }
}

fn check_range(
    value: f64,
    prop: &PropertySymbol,
    entity_id: &str,
    span: &Span,
    diagnostics: &mut DiagnosticCollector,
) -> bool {
    let min = prop.min.unwrap_or(f64::NEG_INFINITY);
    let max = prop.max.unwrap_or(f64::INFINITY);
    if prop.min.is_some() || prop.max.is_some() {
        if value < min || value > max {
            let min_str = prop.min.map(|v| v.to_string()).unwrap_or_else(|| "-∞".to_string());
            let max_str = prop.max.map(|v| v.to_string()).unwrap_or_else(|| "∞".to_string());
            diagnostics.error(
                "URD418",
                format!(
                    "Value {} for property '{}' on entity '@{}' is outside the declared range [{}, {}].",
                    value, prop.name, entity_id, min_str, max_str,
                ),
                span.clone(),
            );
            return false;
        }
    }
    true
}

fn check_ref_type(
    ref_id: &str,
    prop: &PropertySymbol,
    entity_id: &str,
    symbol_table: &SymbolTable,
    span: &Span,
    diagnostics: &mut DiagnosticCollector,
) {
    if let Some(expected_type) = &prop.ref_type {
        if let Some(ref_entity) = symbol_table.entities.get(ref_id) {
            if let Some(actual_type) = &ref_entity.type_symbol {
                if actual_type != expected_type {
                    diagnostics.error(
                        "URD419",
                        format!(
                            "Property '{}' on entity '@{}' requires a reference to type '{}' but '@{}' has type '{}'.",
                            prop.name, entity_id, expected_type, ref_id, actual_type,
                        ),
                        span.clone(),
                    );
                }
            }
            // If ref_entity.type_symbol is None, skip — LINK already reported URD307.
        }
        // If ref_id not in entities, skip — LINK already reported URD301.
    }
}
