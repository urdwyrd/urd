/// Phase 2: IMPORT — entry file AST to dependency graph.
///
/// Input:  Entry `FileAst`
/// Output: `DependencyGraph` + all `FileAst`s in topological order
///
/// Key guarantee: acyclic, depth-limited, file stems unique, paths normalised.
///
/// Diagnostic code range: URD200–URD299

use crate::ast::FileAst;
use crate::diagnostics::DiagnosticCollector;
use crate::graph::DependencyGraph;

/// Resolve all imports starting from the entry file AST.
///
/// Discovers imported files recursively, parses each via PARSE, builds
/// the dependency graph, and produces a topologically sorted file list.
pub fn resolve_imports(_entry_ast: FileAst, _diagnostics: &mut DiagnosticCollector) -> DependencyGraph {
    // Stub: phase logic will be implemented per the IMPORT phase brief.
    todo!("IMPORT phase not yet implemented")
}
