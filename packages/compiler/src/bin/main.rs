/// Urd compiler CLI — reads a `.urd.md` file and writes `.urd.json` to stdout.
///
/// Usage: urd <file.urd.md>
///
/// Diagnostics are printed to stderr. The compiled JSON is printed to stdout
/// on success. Exit code 0 on success, 1 on compilation errors.

use urd_compiler::import::OsFileReader;

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: urd <file.urd.md>");
        std::process::exit(1);
    });

    let source = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("Cannot read '{}': {}", path, e);
        std::process::exit(1);
    });

    let result = urd_compiler::compile_source_with_reader(&path, &source, &OsFileReader);

    // Print diagnostics to stderr.
    for d in result.diagnostics.sorted() {
        let severity = match d.severity {
            urd_compiler::diagnostics::Severity::Error => "error",
            urd_compiler::diagnostics::Severity::Warning => "warning",
            urd_compiler::diagnostics::Severity::Info => "info",
        };
        eprintln!("[{}] {}: {} ({})", severity, d.span, d.message, d.code);
    }

    if let Some(json) = result.world {
        println!("{}", json);
    } else {
        std::process::exit(1);
    }
}
