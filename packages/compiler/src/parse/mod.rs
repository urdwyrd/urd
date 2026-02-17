/// Phase 1: PARSE — source text to per-file ASTs.
///
/// Input:  `.urd.md` source text
/// Output: `FileAst` with span-tracked nodes
///
/// Key guarantee: every syntactically valid construct has a node.
/// Errors produce `ErrorNode` markers. The parser never aborts.
///
/// Diagnostic code range: URD100–URD199

use crate::ast::FileAst;
use crate::diagnostics::DiagnosticCollector;
use crate::span::FilePath;

/// Parse a single `.urd.md` source file into a `FileAst`.
///
/// Returns `None` only if the file cannot be read at all.
/// Syntax errors produce `ErrorNode` markers in the AST and diagnostics
/// in the collector — the parser always produces a (partial) AST.
pub fn parse(_path: &FilePath, _source: &str, _diagnostics: &mut DiagnosticCollector) -> Option<FileAst> {
    // Stub: phase logic will be implemented per the PARSE phase brief.
    todo!("PARSE phase not yet implemented")
}
