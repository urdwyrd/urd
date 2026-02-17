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

use crate::diagnostics::DiagnosticCollector;
use crate::graph::DependencyGraph;
use crate::symbol_table::SymbolTable;

/// Emit the compiled `.urd.json` string from the validated world.
///
/// Precondition: `diagnostics.has_errors()` is `false`.
pub fn emit(
    _graph: &DependencyGraph,
    _symbol_table: &SymbolTable,
    _diagnostics: &mut DiagnosticCollector,
) -> String {
    // Stub: will be implemented per the EMIT phase brief.
    todo!("EMIT phase not yet implemented")
}
