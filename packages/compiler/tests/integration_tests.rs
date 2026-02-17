/// End-to-end integration tests for the full compiler pipeline.
///
/// These tests exercise the complete `compile()` orchestrator function,
/// verifying that source files are correctly transformed through all
/// five phases into valid `.urd.json` output.

use urd_compiler::CompilationResult;

/// Helper: verify a compilation result has no errors.
#[allow(dead_code)]
fn assert_success(result: &CompilationResult) {
    assert!(
        result.success,
        "Expected successful compilation, but got errors: {:?}",
        result.diagnostics.sorted().iter().map(|d| &d.message).collect::<Vec<_>>()
    );
    assert!(result.world.is_some(), "Expected JSON output on success");
}

/// Helper: verify a compilation result has errors.
#[allow(dead_code)]
fn assert_failure(result: &CompilationResult) {
    assert!(!result.success, "Expected compilation failure, but got success");
    assert!(result.world.is_none(), "Expected no JSON output on failure");
}

// Placeholder integration test cases:
//
// Two Room Key Puzzle (from the architecture brief worked example):
//   - Single-file world with types, entities, locations, choices, conditions, effects
//   - Output matches expected JSON structure
//   - Zero diagnostics
//
// Multi-file import:
//   - Entry file imports shared types file
//   - Types from imported file visible in entry file
//   - Topological ordering reflected in output
//
// Error scenarios:
//   - File not found → error diagnostic, no JSON
//   - Syntax error → partial AST, error diagnostic
//   - Unresolved reference → error diagnostic, no JSON
//   - Type mismatch → error diagnostic, no JSON
//   - Circular import → error diagnostic, compilation continues
//
// Determinism:
//   - Compile same source twice → byte-identical output
