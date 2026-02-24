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
pub mod facts;
pub mod diff;
pub mod analyze;
pub mod slugify;
pub mod symbol_table;

// Phase modules
pub mod parse;
pub mod import;
pub mod link;
pub mod validate;
pub mod emit;

// WASM bindings (only compiled with the `wasm` feature)
#[cfg(feature = "wasm")]
pub mod wasm;

use diagnostics::DiagnosticCollector;
use import::FileReader;
#[cfg(not(target_arch = "wasm32"))]
use span::FilePath;

/// The result of a compilation.
pub struct CompilationResult {
    /// `true` if compilation succeeded with zero errors.
    pub success: bool,
    /// The compiled JSON string, or `None` if any errors occurred.
    pub world: Option<String>,
    /// All diagnostics (errors, warnings, info) from all phases.
    pub diagnostics: DiagnosticCollector,
    /// Normalized analysis IR. `Some` whenever LINK succeeds, even if
    /// VALIDATE emits errors. `None` only on PARSE or IMPORT failure.
    pub fact_set: Option<facts::FactSet>,
    /// Property dependency index built from the FactSet. `Some` whenever
    /// `fact_set` is `Some`. Used by ANALYZE diagnostics and WASM serialisation.
    pub property_index: Option<facts::PropertyDependencyIndex>,
}

/// Compile a single `.urd.md` source string (no import resolution).
///
/// Import declarations in the source will produce URD201 diagnostics
/// (file not found) because this function runs in single-file mode.
/// For multi-file compilation, use [`compile_source_with_reader()`].
pub fn compile_source(filename: &str, source: &str) -> CompilationResult {
    compile_source_with_reader(filename, source, &import::StubFileReader)
}

/// Compile a `.urd.md` source string with a custom file reader.
///
/// The reader is used to resolve `import:` declarations. Native callers
/// pass [`import::OsFileReader`]; WASM callers pass a stub or
/// JavaScript-backed reader.
///
/// Orchestrates the five phases in sequence:
/// 1. PARSE    — source text → per-file AST
/// 2. IMPORT   — entry AST → dependency graph + all ASTs
/// 3. LINK     — graph + ASTs → symbol table + annotated ASTs
/// 4. VALIDATE — annotated ASTs + symbol table → diagnostics
/// 5. EMIT     — validated ASTs + symbol table → `.urd.json`
pub fn compile_source_with_reader(
    filename: &str,
    source: &str,
    reader: &dyn FileReader,
) -> CompilationResult {
    let mut diagnostics = DiagnosticCollector::new();

    // Normalise filename: split into directory + filename components.
    let normalised = filename.replace('\\', "/");
    let (entry_dir, entry_filename) = match normalised.rfind('/') {
        Some(pos) => (
            normalised[..pos + 1].to_string(),
            normalised[pos + 1..].to_string(),
        ),
        None => (String::new(), normalised),
    };

    // Phase 1: PARSE
    let entry_ast = match parse::parse(&entry_filename, source, &mut diagnostics) {
        Some(ast) => ast,
        None => {
            return CompilationResult {
                success: false,
                world: None,
                diagnostics,
                fact_set: None,
                property_index: None,
            };
        }
    };

    // Phase 2: IMPORT
    let compilation_unit =
        import::resolve_imports_with_reader(entry_ast, &entry_dir, &mut diagnostics, reader);

    // Fatal IMPORT errors (URD203, URD205) prevent LINK.
    if diagnostics.has_errors() {
        return CompilationResult {
            success: false,
            world: None,
            diagnostics,
            fact_set: None,
            property_index: None,
        };
    }

    // Phase 3: LINK
    let linked = link::link(compilation_unit, &mut diagnostics);

    // Phase 3a: Extract facts (always succeeds when LINK completes)
    let fact_set = Some(facts::extract_facts(&linked.graph, &linked.symbol_table));

    // Phase 3b: Build property dependency index
    let property_index = fact_set
        .as_ref()
        .map(facts::PropertyDependencyIndex::build);

    // Phase 3c: ANALYZE (FactSet-derived diagnostics, URD600–URD699)
    if let (Some(ref fs), Some(ref idx)) = (&fact_set, &property_index) {
        for diag in analyze::analyze(fs, idx) {
            diagnostics.emit(diag);
        }
    }

    // Phase 4: VALIDATE
    validate::validate(&linked.graph, &linked.symbol_table, &mut diagnostics);

    // Phase 5: EMIT
    if diagnostics.has_errors() {
        return CompilationResult {
            success: false,
            world: None,
            diagnostics,
            fact_set,
            property_index,
        };
    }

    let json = emit::emit(&linked.graph, &linked.symbol_table, &mut diagnostics);

    CompilationResult {
        success: true,
        world: Some(json),
        diagnostics,
        fact_set,
        property_index,
    }
}

/// Convenience: compile from a file path (reads the file from disk).
///
/// Equivalent to reading the file and calling [`compile_source_with_reader()`]
/// with [`import::OsFileReader`].
///
/// Not available on WASM targets.
#[cfg(not(target_arch = "wasm32"))]
pub fn compile(entry_file: &FilePath) -> CompilationResult {
    let mut diagnostics = DiagnosticCollector::new();

    let normalised = entry_file.replace('\\', "/");
    let entry_filename = match normalised.rfind('/') {
        Some(pos) => normalised[pos + 1..].to_string(),
        None => normalised.clone(),
    };

    let source = match std::fs::read_to_string(entry_file) {
        Ok(s) => s,
        Err(e) => {
            diagnostics.error(
                "URD100",
                format!("Cannot read file '{}': {}", entry_file, e),
                span::Span::new(entry_filename, 1, 1, 1, 1),
            );
            return CompilationResult {
                success: false,
                world: None,
                diagnostics,
                fact_set: None,
                property_index: None,
            };
        }
    };

    compile_source_with_reader(entry_file, &source, &import::OsFileReader)
}
