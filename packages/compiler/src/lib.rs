/// Urd Compiler — five-phase pipeline from `.urd.md` to `.urd.json`.
///
/// ```text
/// .urd.md → PARSE → IMPORT → LINK → VALIDATE → EMIT → .urd.json
/// ```
///
/// Each phase has a defined input contract, output contract, and set of
/// guarantees. See the architecture brief for the full specification.

pub mod ast;
pub mod diagnostics;
pub mod graph;
pub mod span;
pub mod slugify;
pub mod symbol_table;

// Phase modules
pub mod parse;
pub mod import;
pub mod link;
pub mod validate;
pub mod emit;

use diagnostics::DiagnosticCollector;
use span::FilePath;

/// The result of a `compile()` call.
#[derive(Debug)]
pub struct CompilationResult {
    /// `true` if compilation succeeded with zero errors.
    pub success: bool,
    /// The compiled JSON string, or `None` if any errors occurred.
    pub world: Option<String>,
    /// All diagnostics (errors, warnings, info) from all phases.
    pub diagnostics: DiagnosticCollector,
}

/// Top-level compiler entry point.
///
/// Orchestrates the five phases in sequence:
/// 1. PARSE  — source text → per-file AST
/// 2. IMPORT — entry AST → dependency graph + all ASTs
/// 3. LINK   — graph + ASTs → symbol table + annotated ASTs
/// 4. VALIDATE — annotated ASTs + symbol table → diagnostics
/// 5. EMIT   — validated ASTs + symbol table → `.urd.json`
pub fn compile(entry_file: &FilePath) -> CompilationResult {
    let mut diagnostics = DiagnosticCollector::new();

    // Phase 1: PARSE
    let source = match std::fs::read_to_string(entry_file) {
        Ok(s) => s,
        Err(e) => {
            diagnostics.error(
                "URD100",
                format!("Cannot read file '{}': {}", entry_file, e),
                span::Span::new(entry_file.clone(), 1, 1, 1, 1),
            );
            return CompilationResult {
                success: false,
                world: None,
                diagnostics,
            };
        }
    };

    let entry_ast = match parse::parse(entry_file, &source, &mut diagnostics) {
        Some(ast) => ast,
        None => {
            return CompilationResult {
                success: false,
                world: None,
                diagnostics,
            };
        }
    };

    // Phase 2: IMPORT
    let graph = import::resolve_imports(entry_ast, &mut diagnostics);

    // Phase 3: LINK
    let mut symbol_table = symbol_table::SymbolTable::default();

    // Pass 1: Collection — register all declarations
    link::collect_declarations(&graph, &mut symbol_table, &mut diagnostics);

    // Pass 2: Resolution — resolve all references
    link::resolve_references(&graph, &symbol_table, &mut diagnostics);

    // Phase 4: VALIDATE
    validate::validate(&graph, &symbol_table, &mut diagnostics);

    // Phase 5: EMIT
    if diagnostics.has_errors() {
        return CompilationResult {
            success: false,
            world: None,
            diagnostics,
        };
    }

    let json = emit::emit(&graph, &symbol_table, &mut diagnostics);

    CompilationResult {
        success: true,
        world: Some(json),
        diagnostics,
    }
}
