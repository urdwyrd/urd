/// Benchmark harness for the Urd compiler.
///
/// Compiles a single `.urd.md` file, timing each phase independently,
/// and prints a JSON line to stdout with per-phase timings.
///
/// Usage: bench <file.urd.md>

use std::time::Instant;

use urd_compiler::diagnostics::DiagnosticCollector;
use urd_compiler::{emit, import, link, parse, validate};

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: bench <file.urd.md>");
        std::process::exit(1);
    });

    let source = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("Cannot read '{}': {}", path, e);
        std::process::exit(1);
    });
    let source_bytes = source.len();

    // Normalise path — same logic as compile() in lib.rs.
    let normalised = path.replace('\\', "/");
    let (entry_dir, entry_filename) = match normalised.rfind('/') {
        Some(pos) => (
            normalised[..pos + 1].to_string(),
            normalised[pos + 1..].to_string(),
        ),
        None => (String::new(), normalised.clone()),
    };

    let mut diagnostics = DiagnosticCollector::new();
    let total_start = Instant::now();

    // Phase 1: PARSE
    let t = Instant::now();
    let ast = match parse::parse(&entry_filename, &source, &mut diagnostics) {
        Some(ast) => ast,
        None => {
            print_result(&normalised, source_bytes, 0, &diagnostics, total_start, 0.0, 0.0, 0.0, 0.0, 0.0, false);
            return;
        }
    };
    let parse_ms = t.elapsed().as_secs_f64() * 1000.0;

    // Phase 2: IMPORT
    let t = Instant::now();
    let compilation_unit = import::resolve_imports(ast, &entry_dir, &mut diagnostics);
    let import_ms = t.elapsed().as_secs_f64() * 1000.0;

    if diagnostics.has_errors() {
        print_result(&normalised, source_bytes, 0, &diagnostics, total_start, parse_ms, import_ms, 0.0, 0.0, 0.0, false);
        return;
    }

    // Phase 3: LINK
    let t = Instant::now();
    let linked = link::link(compilation_unit, &mut diagnostics);
    let link_ms = t.elapsed().as_secs_f64() * 1000.0;

    // Phase 4: VALIDATE
    let t = Instant::now();
    validate::validate(&linked.graph, &linked.symbol_table, &mut diagnostics);
    let validate_ms = t.elapsed().as_secs_f64() * 1000.0;

    // Phase 5: EMIT
    let (emit_ms, output_bytes) = if diagnostics.has_errors() {
        (0.0, 0)
    } else {
        let t = Instant::now();
        let output = emit::emit(&linked.graph, &linked.symbol_table, &mut diagnostics);
        (t.elapsed().as_secs_f64() * 1000.0, output.len())
    };

    let success = !diagnostics.has_errors();
    print_result(&normalised, source_bytes, output_bytes, &diagnostics, total_start, parse_ms, import_ms, link_ms, validate_ms, emit_ms, success);
}

fn print_result(
    path: &str,
    source_bytes: usize,
    output_bytes: usize,
    diagnostics: &DiagnosticCollector,
    total_start: Instant,
    parse_ms: f64,
    import_ms: f64,
    link_ms: f64,
    validate_ms: f64,
    emit_ms: f64,
    success: bool,
) {
    let total_ms = total_start.elapsed().as_secs_f64() * 1000.0;

    // Derive name from file path: strip directory and extension.
    // For multi-file fixtures (e.g. interrogation/main.urd.md), use the
    // parent directory name instead of the generic "main" filename.
    let parts: Vec<&str> = path.split('/').collect();
    let filename = parts.last().unwrap_or(&path).trim_end_matches(".urd.md");
    let name = if filename == "main" && parts.len() >= 2 {
        parts[parts.len() - 2].replace('-', "_")
    } else {
        filename.replace('-', "_")
    };

    let diagnostic_count = diagnostics.len();

    let result = serde_json::json!({
        "name": name,
        "source_bytes": source_bytes,
        "output_bytes": output_bytes,
        "total_ms": round3(total_ms),
        "parse_ms": round3(parse_ms),
        "import_ms": round3(import_ms),
        "link_ms": round3(link_ms),
        "validate_ms": round3(validate_ms),
        "emit_ms": round3(emit_ms),
        "success": success,
        "diagnostic_count": diagnostic_count,
    });

    println!("{}", result);
}

/// Round to 3 decimal places for readable output.
fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}
