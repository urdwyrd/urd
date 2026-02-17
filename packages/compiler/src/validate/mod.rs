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

use crate::diagnostics::DiagnosticCollector;
use crate::graph::DependencyGraph;
use crate::symbol_table::SymbolTable;

/// Validate the linked world: type-check properties, conditions, effects.
/// Enforce all semantic constraints defined in the spec.
pub fn validate(
    _graph: &DependencyGraph,
    _symbol_table: &SymbolTable,
    _diagnostics: &mut DiagnosticCollector,
) {
    // Stub: will be implemented per the VALIDATE phase brief.
    todo!("VALIDATE phase not yet implemented")
}
