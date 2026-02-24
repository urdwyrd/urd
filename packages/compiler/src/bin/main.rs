/// Urd compiler CLI — compile, diff, and snapshot `.urd.md` files.
///
/// Usage:
///   urd <file.urd.md>                         Compile and emit .urd.json to stdout
///   urd diff <a> <b> [--format json|summary]  Compare two files and report changes
///   urd snapshot <file.urd.md> [-o output]    Create a .urd.snapshot.json
///
/// Diagnostics are printed to stderr. Exit code 0 on success (or no changes),
/// 1 on errors (or changes detected by diff).

use urd_compiler::diff::{DiffSnapshot, DiffError};
use urd_compiler::import::OsFileReader;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("diff") => run_diff(&args[2..]),
        Some("snapshot") => run_snapshot(&args[2..]),
        Some(path) if !path.starts_with('-') => run_compile(path),
        _ => print_usage(),
    }
}

fn print_usage() {
    eprintln!("Urd compiler");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  urd <file.urd.md>                         Compile to .urd.json (stdout)");
    eprintln!("  urd diff <a> <b> [--format json|summary]  Compare two compilations");
    eprintln!("  urd snapshot <file.urd.md> [-o output]    Create a snapshot file");
    std::process::exit(1);
}

// ── Compile (default command) ──

fn run_compile(path: &str) {
    let source = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Cannot read '{}': {}", path, e);
        std::process::exit(1);
    });

    let result = urd_compiler::compile_source_with_reader(path, &source, &OsFileReader);
    print_diagnostics(&result);

    if let Some(json) = result.world {
        println!("{}", json);
    } else {
        std::process::exit(1);
    }
}

// ── Diff command ──

fn run_diff(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: urd diff <file_a> <file_b> [--format json|summary]");
        std::process::exit(1);
    }

    let path_a = &args[0];
    let path_b = &args[1];

    // Parse --format flag.
    let mut format = "json";
    let mut i = 2;
    while i < args.len() {
        if args[i] == "--format" && i + 1 < args.len() {
            format = match args[i + 1].as_str() {
                "json" => "json",
                "summary" => "summary",
                other => {
                    eprintln!("Unknown format '{}'. Use 'json' or 'summary'.", other);
                    std::process::exit(1);
                }
            };
            i += 2;
        } else {
            eprintln!("Unknown argument '{}'", args[i]);
            std::process::exit(1);
        }
    }

    let snap_a = load_snapshot(path_a);
    let snap_b = load_snapshot(path_b);
    let report = urd_compiler::diff::diff(&snap_a, &snap_b);

    match format {
        "summary" => {
            println!("{}", report.summary());
        }
        _ => {
            let json = serde_json::to_string_pretty(&report.to_json()).unwrap();
            println!("{}", json);
        }
    }

    if report.changes.is_empty() {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

// ── Snapshot command ──

fn run_snapshot(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: urd snapshot <file.urd.md> [-o output.snapshot.json]");
        std::process::exit(1);
    }

    let path = &args[0];

    // Parse -o flag.
    let mut output_path: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "-o" && i + 1 < args.len() {
            output_path = Some(args[i + 1].clone());
            i += 2;
        } else {
            eprintln!("Unknown argument '{}'", args[i]);
            std::process::exit(1);
        }
    }

    let output = output_path.unwrap_or_else(|| {
        let stem = path
            .strip_suffix(".urd.md")
            .unwrap_or(path);
        format!("{}.urd.snapshot.json", stem)
    });

    let source = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Cannot read '{}': {}", path, e);
        std::process::exit(1);
    });

    let result = urd_compiler::compile_source_with_reader(path, &source, &OsFileReader);
    print_diagnostics(&result);

    if result.diagnostics.has_errors() && result.fact_set.is_none() {
        eprintln!("Compilation failed; cannot create snapshot.");
        std::process::exit(1);
    }

    // Extract world name from compiled JSON.
    let world_name = result
        .world
        .as_deref()
        .and_then(|w| serde_json::from_str::<serde_json::Value>(w).ok())
        .and_then(|v| v.get("world")?.get("name")?.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    let snapshot = DiffSnapshot::from_compilation(&result);
    let json = serde_json::to_string_pretty(&snapshot.to_json(&world_name)).unwrap();

    std::fs::write(&output, format!("{}\n", json)).unwrap_or_else(|e| {
        eprintln!("Cannot write '{}': {}", output, e);
        std::process::exit(1);
    });

    eprintln!("Snapshot written to {}", output);
}

// ── Helpers ──

/// Load a DiffSnapshot from either a .urd.md source or a .urd.snapshot.json file.
fn load_snapshot(path: &str) -> DiffSnapshot {
    if path.ends_with(".urd.snapshot.json") {
        let content = std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("Cannot read '{}': {}", path, e);
            std::process::exit(1);
        });
        match DiffSnapshot::from_json(&content) {
            Ok(snap) => snap,
            Err(DiffError::UnsupportedSnapshotVersion) => {
                eprintln!("Unsupported snapshot version in '{}'. Regenerate with current compiler.", path);
                std::process::exit(1);
            }
            Err(DiffError::ParseError(msg)) => {
                eprintln!("Failed to parse snapshot '{}': {}", path, msg);
                std::process::exit(1);
            }
        }
    } else {
        let source = std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("Cannot read '{}': {}", path, e);
            std::process::exit(1);
        });
        let result = urd_compiler::compile_source_with_reader(path, &source, &OsFileReader);
        print_diagnostics(&result);
        DiffSnapshot::from_compilation(&result)
    }
}

fn print_diagnostics(result: &urd_compiler::CompilationResult) {
    for d in result.diagnostics.sorted() {
        let severity = match d.severity {
            urd_compiler::diagnostics::Severity::Error => "error",
            urd_compiler::diagnostics::Severity::Warning => "warning",
            urd_compiler::diagnostics::Severity::Info => "info",
        };
        eprintln!("[{}] {}: {} ({})", severity, d.span, d.message, d.code);
    }
}
