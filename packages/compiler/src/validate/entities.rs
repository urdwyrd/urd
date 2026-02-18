/// Step 3: Entity Property Override Validation
///
/// For each EntitySymbol where type_symbol is resolved:
/// - Type-check each property override value (URD401)
/// - Check enum values (URD402)
/// - Check range constraints (URD418)
/// - Check ref type matches (URD419)

use crate::diagnostics::DiagnosticCollector;
use crate::symbol_table::SymbolTable;

use super::helpers::{check_value, CheckContext};

/// Validate all entity property overrides.
pub fn validate_entities(
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    for (entity_id, entity_sym) in &symbol_table.entities {
        // Skip if type unresolved — LINK already reported URD307.
        let type_name = match &entity_sym.type_symbol {
            Some(t) => t,
            None => continue,
        };
        let type_sym = match symbol_table.types.get(type_name) {
            Some(t) => t,
            None => continue,
        };

        for (prop_name, value) in &entity_sym.property_overrides {
            // Skip if property doesn't exist on type — LINK already reported URD308.
            let prop = match type_sym.properties.get(prop_name) {
                Some(p) => p,
                None => continue,
            };

            check_value(
                value,
                prop,
                entity_id,
                type_name,
                CheckContext::Override,
                symbol_table,
                &entity_sym.declared_in,
                diagnostics,
            );
        }
    }
}
