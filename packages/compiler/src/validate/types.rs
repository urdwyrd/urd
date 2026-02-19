/// Step 2: Type Definition Validation
///
/// For each TypeSymbol in insertion order:
/// - Validate property defaults against declared type (URD413)
/// - Check for empty enum values lists (URD414)
/// - Verify ref_type references exist (URD415)
/// - Check range validity: min ≤ max (URD416)
/// - Check range type compatibility (URD417)

use crate::diagnostics::DiagnosticCollector;
use crate::symbol_table::{PropertyType, SymbolTable};

use super::helpers::{check_value, format_property_type, CheckContext};

/// Validate all type definitions in the symbol table.
pub fn validate_types(
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    for (type_name, type_sym) in &symbol_table.types {
        for (_prop_name, prop) in &type_sym.properties {
            // a. Property defaults.
            if let Some(default) = &prop.default {
                check_value(
                    default,
                    prop,
                    type_name,  // entity_id context — for type defaults, use type name
                    type_name,
                    CheckContext::Default,
                    symbol_table,
                    &type_sym.declared_in,
                    diagnostics,
                );
            }

            // b. Empty enum values list.
            if prop.property_type == PropertyType::Enum {
                if let Some(values) = &prop.values {
                    if values.is_empty() {
                        diagnostics.error(
                            "URD414",
                            format!(
                                "Enum property '{}' on type '{}' declares an empty values list.",
                                prop.name, type_name,
                            ),
                            type_sym.declared_in.clone(),
                        );
                    }
                }
            }

            // c. Ref type existence.
            if prop.property_type == PropertyType::Ref {
                if let Some(ref_type) = &prop.ref_type {
                    if !symbol_table.types.contains_key(ref_type) {
                        diagnostics.error(
                            "URD415",
                            format!(
                                "Property '{}' on type '{}' references unknown type '{}'.",
                                prop.name, type_name, ref_type,
                            ),
                            type_sym.declared_in.clone(),
                        );
                    }
                }
            }

            // d. Range validity.
            if let (Some(min), Some(max)) = (prop.min, prop.max) {
                if min > max {
                    diagnostics.error(
                        "URD416",
                        format!(
                            "Property '{}' on type '{}' has min ({}) greater than max ({}).",
                            prop.name, type_name, min, max,
                        ),
                        type_sym.declared_in.clone(),
                    );
                }
            }

            // e. Range type compatibility.
            if prop.min.is_some() || prop.max.is_some() {
                if !matches!(prop.property_type, PropertyType::Integer | PropertyType::Number) {
                    diagnostics.error(
                        "URD417",
                        format!(
                            "Range constraints (min/max) are only valid on integer and number properties, not '{}'.",
                            format_property_type(&prop.property_type),
                        ),
                        type_sym.declared_in.clone(),
                    );
                }
            }

            // f. Unrecognised type string.
            if !is_recognised_type(&prop.raw_type_string) {
                diagnostics.warning(
                    "URD429",
                    format!(
                        "Unrecognised property type '{}' on property '{}' of type '{}'. \
                         Valid types: bool, int, num, str, enum, ref, list (and their long forms). \
                         Treating as 'string'.",
                        prop.raw_type_string, prop.name, type_name,
                    ),
                    type_sym.declared_in.clone(),
                );
            }
        }
    }
}

/// Check whether a raw type string is a recognised type name.
fn is_recognised_type(s: &str) -> bool {
    matches!(
        s,
        "bool" | "boolean"
            | "int" | "integer"
            | "num" | "number"
            | "str" | "string"
            | "enum"
            | "ref"
            | "list"
    )
}
