/// Compiled world state container for MCP queries.
///
/// Built once from a CompilationResult at startup. Immutable for the
/// server's lifetime. All query functions operate against this struct.

use urd_compiler::diagnostics::Severity;
use urd_compiler::facts::{FactSet, PropertyDependencyIndex};
use urd_compiler::CompilationResult;

/// Serialisable diagnostic entry, flattened from the compiler's Diagnostic.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DiagnosticEntry {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub file: String,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

/// Immutable compiled world data for MCP queries.
pub struct WorldData {
    /// Parsed world JSON from EMIT. `None` if compilation had errors.
    pub world_json: Option<serde_json::Value>,
    /// FactSet from LINK. `None` only on PARSE or IMPORT failure.
    pub fact_set: Option<FactSet>,
    /// Property dependency analysis. `Some` whenever `fact_set` is `Some`.
    pub property_index: Option<PropertyDependencyIndex>,
    /// All diagnostics from compilation, flattened and sorted.
    pub diagnostics: Vec<DiagnosticEntry>,
    /// Whether compilation produced any errors.
    pub has_errors: bool,
}

impl WorldData {
    /// Build WorldData from a CompilationResult.
    ///
    /// Moves owned data out of the result. The CompilationResult is consumed.
    pub fn from_result(result: CompilationResult) -> Self {
        let world_json = result.world.as_ref().and_then(|json_str| {
            serde_json::from_str::<serde_json::Value>(json_str).ok()
        });

        let diagnostics: Vec<DiagnosticEntry> = result
            .diagnostics
            .sorted()
            .iter()
            .map(|d| DiagnosticEntry {
                severity: match d.severity {
                    Severity::Error => "error".to_string(),
                    Severity::Warning => "warning".to_string(),
                    Severity::Info => "info".to_string(),
                },
                code: d.code.clone(),
                message: d.message.clone(),
                file: d.span.file.clone(),
                start_line: d.span.start_line,
                start_col: d.span.start_col,
                end_line: d.span.end_line,
                end_col: d.span.end_col,
            })
            .collect();

        let has_errors = result.diagnostics.has_errors();

        Self {
            world_json,
            fact_set: result.fact_set,
            property_index: result.property_index,
            diagnostics,
            has_errors,
        }
    }
}
