/// WASM bindings for the Urd compiler.
///
/// Provides three entry points for browser/playground use:
/// - `compile_source()` — full five-phase pipeline
/// - `parse_only()` — phase 1 only (for live syntax checking)
/// - `compiler_version()` — crate version string

use wasm_bindgen::prelude::*;

use crate::diagnostics::Severity;

/// Compile a `.urd.md` source string through the full pipeline.
///
/// Returns a JSON string with shape:
/// ```json
/// { "success": bool, "world": string|null, "diagnostics": [...] }
/// ```
#[wasm_bindgen]
pub fn compile_source(source: &str) -> String {
    let result = crate::compile_source("playground.urd.md", source);
    serialise_result(result)
}

/// Run only the PARSE phase (for live syntax checking in the editor).
///
/// Returns a JSON string with shape:
/// ```json
/// { "success": bool, "diagnostics": [...] }
/// ```
#[wasm_bindgen]
pub fn parse_only(source: &str) -> String {
    let mut diagnostics = crate::diagnostics::DiagnosticCollector::new();
    let filename = "playground.urd.md".to_string();
    let parsed = crate::parse::parse(&filename, source, &mut diagnostics);

    let success = parsed.is_some() && !diagnostics.has_errors();
    let diags = serialise_diagnostics(&diagnostics);

    serde_json::json!({
        "success": success,
        "diagnostics": diags,
    })
    .to_string()
}

/// Return the compiler version (from Cargo.toml).
#[wasm_bindgen]
pub fn compiler_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn serialise_result(result: crate::CompilationResult) -> String {
    let diags = serialise_diagnostics(&result.diagnostics);

    serde_json::json!({
        "success": result.success,
        "world": result.world,
        "diagnostics": diags,
    })
    .to_string()
}

fn serialise_diagnostics(collector: &crate::diagnostics::DiagnosticCollector) -> Vec<serde_json::Value> {
    collector
        .sorted()
        .iter()
        .map(|d| {
            serde_json::json!({
                "severity": match d.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                    Severity::Info => "info",
                },
                "code": d.code,
                "message": d.message,
                "span": {
                    "file": d.span.file,
                    "start_line": d.span.start_line,
                    "start_col": d.span.start_col,
                    "end_line": d.span.end_line,
                    "end_col": d.span.end_col,
                },
            })
        })
        .collect()
}
