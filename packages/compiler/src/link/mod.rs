/// Phase 3: LINK — dependency graph + ASTs to symbol table + annotated ASTs.
///
/// Input:  `DependencyGraph` + `FileAst`s
/// Output: `SymbolTable` + annotated ASTs (`LinkedWorld`)
///
/// The most complex phase. Two sequential passes over all files in
/// topological order:
///   Pass 1 (collection): register every declaration.
///   Pass 2 (resolution): resolve every reference, fill annotation slots.
///
/// Key guarantee: every declared name registered, every resolvable
/// reference annotated, duplicates flagged, visible scope enforced.
///
/// Diagnostic code range: URD300–URD399

use crate::diagnostics::DiagnosticCollector;
use crate::graph::DependencyGraph;
use crate::symbol_table::SymbolTable;

/// The output of the LINK phase: annotated ASTs + populated symbol table.
#[derive(Debug)]
pub struct LinkedWorld {
    pub graph: DependencyGraph,
    pub symbol_table: SymbolTable,
}

/// Pass 1: Walk every AST and register every declaration in the symbol table.
pub fn collect_declarations(
    _graph: &DependencyGraph,
    _symbol_table: &mut SymbolTable,
    _diagnostics: &mut DiagnosticCollector,
) {
    // Stub: will be implemented per the LINK phase brief.
    todo!("LINK collection sub-pass not yet implemented")
}

/// Pass 2: Walk every AST again, resolve all references, fill annotations.
pub fn resolve_references(
    _graph: &DependencyGraph,
    _symbol_table: &SymbolTable,
    _diagnostics: &mut DiagnosticCollector,
) {
    // Stub: will be implemented per the LINK phase brief.
    todo!("LINK resolution sub-pass not yet implemented")
}
