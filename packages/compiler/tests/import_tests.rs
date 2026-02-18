// Tests for Phase 2: IMPORT
//
// Test categories from the IMPORT phase brief:
// - Path resolution (12 tests)
// - Graph construction (11 tests)
// - Topological sort determinism (4 tests)
// - Error recovery (5 tests)
// - Span reference (3 tests)

use std::collections::HashMap;
use urd_compiler::diagnostics::DiagnosticCollector;
use urd_compiler::import::{FileReadError, FileReader};
use urd_compiler::import::resolve_imports_with_reader;
use urd_compiler::parse;

// ── Test helpers ────────────────────────────────────────────────────

/// In-memory filesystem for testing. Maps normalised paths to file contents.
struct MockFs {
    files: HashMap<String, MockFile>,
}

enum MockFile {
    Content(String),
    PermissionDenied,
    InvalidUtf8,
    IoError(String),
}

impl MockFs {
    fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    fn add(mut self, path: &str, content: &str) -> Self {
        self.files
            .insert(path.to_string(), MockFile::Content(content.to_string()));
        self
    }

    fn add_error(mut self, path: &str, error: MockFile) -> Self {
        self.files.insert(path.to_string(), error);
        self
    }
}

impl FileReader for MockFs {
    fn read_file(&self, fs_path: &str) -> Result<String, FileReadError> {
        match self.files.get(fs_path) {
            Some(MockFile::Content(s)) => Ok(s.clone()),
            Some(MockFile::PermissionDenied) => Err(FileReadError::PermissionDenied),
            Some(MockFile::InvalidUtf8) => Err(FileReadError::InvalidUtf8),
            Some(MockFile::IoError(msg)) => Err(FileReadError::IoError(msg.clone())),
            None => Err(FileReadError::NotFound),
        }
    }

    fn canonical_filename(&self, _dir: &str, _filename: &str) -> Option<String> {
        None // No casing checks in mock FS
    }
}

/// Mock FS that detects casing mismatches for specific filenames.
struct CasingMockFs {
    inner: MockFs,
    /// Maps (dir, wrong_filename) → canonical_filename
    casing_map: HashMap<(String, String), String>,
}

impl CasingMockFs {
    fn new(inner: MockFs) -> Self {
        Self {
            inner,
            casing_map: HashMap::new(),
        }
    }

    fn add_casing(mut self, dir: &str, wrong: &str, canonical: &str) -> Self {
        self.casing_map
            .insert((dir.to_string(), wrong.to_string()), canonical.to_string());
        self
    }
}

impl FileReader for CasingMockFs {
    fn read_file(&self, fs_path: &str) -> Result<String, FileReadError> {
        self.inner.read_file(fs_path)
    }

    fn canonical_filename(&self, dir: &str, filename: &str) -> Option<String> {
        self.casing_map
            .get(&(dir.to_string(), filename.to_string()))
            .cloned()
    }
}

/// Build a minimal .urd.md source string with the given imports and content.
fn make_source(imports: &[&str], content: &str) -> String {
    if imports.is_empty() && content.is_empty() {
        return String::new();
    }
    let mut source = String::from("---\n");
    for imp in imports {
        source.push_str(&format!("import: {}\n", imp));
    }
    source.push_str("---\n");
    if !content.is_empty() {
        source.push_str(content);
        if !content.ends_with('\n') {
            source.push('\n');
        }
    }
    source
}

/// Parse a source string into a FileAst.
fn parse_source(path: &str, source: &str) -> urd_compiler::ast::FileAst {
    let mut diag = DiagnosticCollector::new();
    parse::parse(&path.to_string(), source, &mut diag)
        .unwrap_or_else(|| panic!("Failed to parse test source for {}", path))
}

/// Find a diagnostic by code.
fn find_diagnostic<'a>(
    diag: &'a DiagnosticCollector,
    code: &str,
) -> Option<&'a urd_compiler::diagnostics::Diagnostic> {
    diag.all().iter().find(|d| d.code == code)
}

/// Count diagnostics with the given code.
#[allow(dead_code)]
fn count_diagnostics(diag: &DiagnosticCollector, code: &str) -> usize {
    diag.all().iter().filter(|d| d.code == code).count()
}

// ── Path resolution tests ───────────────────────────────────────────

#[test]
fn path_simple_relative() {
    let source = make_source(&["./types.urd.md"], "");
    let ast = parse_source("world.urd.md", &source);
    let fs = MockFs::new().add("types.urd.md", &make_source(&[], ""));
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(!diag.has_errors());
    assert!(graph.nodes.contains_key("types.urd.md"));
    assert_eq!(graph.nodes.len(), 2);
}

#[test]
fn path_subdirectory() {
    let source = make_source(&["./shared/types.urd.md"], "");
    let ast = parse_source("world.urd.md", &source);
    let fs = MockFs::new().add("shared/types.urd.md", &make_source(&[], ""));
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(!diag.has_errors());
    assert!(graph.nodes.contains_key("shared/types.urd.md"));
}

#[test]
fn path_parent_directory() {
    // content/tavern.urd.md imports ../shared/types.urd.md → shared/types.urd.md
    // Entry is at root, tavern is in content/, so ../shared resolves to shared/.
    let entry_source = make_source(&["./content/tavern.urd.md"], "");
    let entry_ast = parse_source("world.urd.md", &entry_source);
    let tavern_source = make_source(&["../shared/types.urd.md"], "");
    let types_source = make_source(&[], "");
    let fs = MockFs::new()
        .add("content/tavern.urd.md", &tavern_source)
        .add("shared/types.urd.md", &types_source);
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(entry_ast, "", &mut diag, &fs);

    assert!(!diag.has_errors(), "Unexpected errors: {:?}", diag.all());
    assert!(graph.nodes.contains_key("shared/types.urd.md"));
}

#[test]
fn path_nested_parent() {
    // content/scenes/tavern.urd.md imports ../../lib/core.urd.md → lib/core.urd.md
    let entry_source = make_source(&["./content/scenes/tavern.urd.md"], "");
    let entry_ast = parse_source("world.urd.md", &entry_source);
    let tavern_source = make_source(&["../../lib/core.urd.md"], "");
    let core_source = make_source(&[], "");
    let fs = MockFs::new()
        .add("content/scenes/tavern.urd.md", &tavern_source)
        .add("lib/core.urd.md", &core_source);
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(entry_ast, "", &mut diag, &fs);

    assert!(!diag.has_errors(), "Unexpected errors: {:?}", diag.all());
    assert!(graph.nodes.contains_key("lib/core.urd.md"));
}

#[test]
fn path_no_leading_dot_slash() {
    let source = make_source(&["types.urd.md"], "");
    let ast = parse_source("world.urd.md", &source);
    let fs = MockFs::new().add("types.urd.md", &make_source(&[], ""));
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(!diag.has_errors());
    assert!(graph.nodes.contains_key("types.urd.md"));
}

#[test]
fn path_backslash_conversion() {
    let source = make_source(&[".\\shared\\types.urd.md"], "");
    let ast = parse_source("world.urd.md", &source);
    let fs = MockFs::new().add("shared/types.urd.md", &make_source(&[], ""));
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(!diag.has_errors());
    assert!(graph.nodes.contains_key("shared/types.urd.md"));
}

#[test]
fn path_outside_project_root() {
    let source = make_source(&["../../outside.urd.md"], "");
    let ast = parse_source("tavern.urd.md", &source);
    let fs = MockFs::new();
    let mut diag = DiagnosticCollector::new();

    let _graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(find_diagnostic(&diag, "URD208").is_some());
}

#[test]
fn path_absolute_unix() {
    let source = make_source(&["/usr/shared/types.urd.md"], "");
    let ast = parse_source("world.urd.md", &source);
    let fs = MockFs::new();
    let mut diag = DiagnosticCollector::new();

    let _graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(find_diagnostic(&diag, "URD209").is_some());
}

#[test]
fn path_absolute_windows() {
    let source = make_source(&["C:\\types.urd.md"], "");
    let ast = parse_source("world.urd.md", &source);
    let fs = MockFs::new();
    let mut diag = DiagnosticCollector::new();

    let _graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    // After backslash normalisation: C:/types.urd.md — absolute path.
    assert!(find_diagnostic(&diag, "URD209").is_some());
}

#[test]
fn path_missing_extension() {
    let source = make_source(&["./types.md"], "");
    let ast = parse_source("world.urd.md", &source);
    let fs = MockFs::new();
    let mut diag = DiagnosticCollector::new();

    let _graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(find_diagnostic(&diag, "URD210").is_some());
}

#[test]
fn path_empty() {
    // Build a source with an empty import value.
    // PARSE stores an ImportDecl with an empty path string.
    // IMPORT should catch it with URD211.
    let source = "---\nimport: \n---\n";
    let ast = parse_source("world.urd.md", source);

    // Check if PARSE produced an ImportDecl with empty path.
    let has_import = ast.frontmatter.as_ref().map_or(false, |fm| {
        fm.entries.iter().any(|e| {
            matches!(&e.value, urd_compiler::ast::FrontmatterValue::ImportDecl(_))
        })
    });

    let fs = MockFs::new();
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    if has_import {
        // PARSE produced an ImportDecl → IMPORT should catch it with URD211.
        assert!(
            find_diagnostic(&diag, "URD211").is_some(),
            "Expected URD211 for empty import path"
        );
    }
    // Either way, no panic and graph is valid.
    assert!(graph_is_valid(&graph));
    assert_eq!(graph.nodes.len(), 1); // Only entry
}

#[test]
fn path_self_import() {
    let source = make_source(&["./world.urd.md"], "");
    let ast = parse_source("world.urd.md", &source);
    let fs = MockFs::new();
    let mut diag = DiagnosticCollector::new();

    let _graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(find_diagnostic(&diag, "URD207").is_some());
}

#[test]
fn path_whitespace_trimmed() {
    let source = make_source(&["  ./types.urd.md  "], "");
    let ast = parse_source("world.urd.md", &source);
    let fs = MockFs::new().add("types.urd.md", &make_source(&[], ""));
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(!diag.has_errors());
    assert!(graph.nodes.contains_key("types.urd.md"));
}

// ── Graph construction tests ────────────────────────────────────────

#[test]
fn graph_single_file_no_imports() {
    let source = make_source(&[], "# Tavern\n");
    let ast = parse_source("entry.urd.md", &source);
    let fs = MockFs::new();
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(!diag.has_errors());
    assert_eq!(graph.nodes.len(), 1);
    assert_eq!(graph.edges.len(), 0);
    let order = graph.topological_order();
    assert_eq!(order.len(), 1);
    assert_eq!(order[0], "entry.urd.md");
}

#[test]
fn graph_linear_chain_a_b_c() {
    // A imports B, B imports C, C has no imports.
    let a_source = make_source(&["./b.urd.md"], "");
    let b_source = make_source(&["./c.urd.md"], "");
    let c_source = make_source(&[], "");
    let ast = parse_source("a.urd.md", &a_source);
    let fs = MockFs::new()
        .add("b.urd.md", &b_source)
        .add("c.urd.md", &c_source);
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(!diag.has_errors());
    assert_eq!(graph.nodes.len(), 3);
    assert_eq!(graph.edges.len(), 2);
    let order: Vec<&str> = graph.topological_order().into_iter().map(|s| s.as_str()).collect();
    assert_eq!(order, vec!["c.urd.md", "b.urd.md", "a.urd.md"]);
}

#[test]
fn graph_diamond_dependency() {
    // A→B, A→C, B→D, C→D — shared dependency D.
    let a_source = make_source(&["./b.urd.md", "./c.urd.md"], "");
    let b_source = make_source(&["./d.urd.md"], "");
    let c_source = make_source(&["./d.urd.md"], "");
    let d_source = make_source(&[], "");
    let ast = parse_source("a.urd.md", &a_source);
    let fs = MockFs::new()
        .add("b.urd.md", &b_source)
        .add("c.urd.md", &c_source)
        .add("d.urd.md", &d_source);
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(!diag.has_errors());
    assert_eq!(graph.nodes.len(), 4);
    // D loaded once (deduplication).
    let order: Vec<&str> = graph.topological_order().into_iter().map(|s| s.as_str()).collect();
    assert_eq!(order, vec!["d.urd.md", "b.urd.md", "c.urd.md", "a.urd.md"]);
}

#[test]
fn graph_duplicate_import() {
    // A imports B twice — B loaded once, one edge, no error.
    let a_source = make_source(&["./b.urd.md", "./b.urd.md"], "");
    let b_source = make_source(&[], "");
    let ast = parse_source("a.urd.md", &a_source);
    let fs = MockFs::new().add("b.urd.md", &b_source);
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(!diag.has_errors());
    assert_eq!(graph.nodes.len(), 2);
    // One edge, not two.
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn graph_cycle_a_b_a() {
    // A imports B, B imports A — cycle detected, B→A skipped.
    let a_source = make_source(&["./b.urd.md"], "");
    let b_source = make_source(&["./a.urd.md"], "");
    let ast = parse_source("a.urd.md", &a_source);
    let fs = MockFs::new().add("b.urd.md", &b_source);
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(find_diagnostic(&diag, "URD202").is_some());
    // Both files in graph — A imported B successfully.
    assert_eq!(graph.nodes.len(), 2);
    // Only one edge: A→B. The B→A edge was rejected.
    assert_eq!(graph.edges.len(), 1);
    let order: Vec<&str> = graph.topological_order().into_iter().map(|s| s.as_str()).collect();
    assert_eq!(order, vec!["b.urd.md", "a.urd.md"]);
}

#[test]
fn graph_longer_cycle_a_b_c_a() {
    // A→B→C→A — three-file cycle.
    let a_source = make_source(&["./b.urd.md"], "");
    let b_source = make_source(&["./c.urd.md"], "");
    let c_source = make_source(&["./a.urd.md"], "");
    let ast = parse_source("a.urd.md", &a_source);
    let fs = MockFs::new()
        .add("b.urd.md", &b_source)
        .add("c.urd.md", &c_source);
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    let d = find_diagnostic(&diag, "URD202").expect("Expected URD202");
    // Cycle path should show the full chain.
    assert!(d.message.contains("a.urd.md"), "Cycle message: {}", d.message);
    assert!(d.message.contains("b.urd.md"), "Cycle message: {}", d.message);
    assert!(d.message.contains("c.urd.md"), "Cycle message: {}", d.message);
    // All three files in graph.
    assert_eq!(graph.nodes.len(), 3);
}

#[test]
fn graph_deep_chain_65_levels() {
    // Chain: entry(1) → file_01(2) → ... → file_63(64) → file_64(65, rejected).
    // The depth check is: when traversal_stack.len() >= 64, reject the import.
    // Entry is at depth 1. file_63 is at depth 64 (the last allowed).
    // file_64 would be at depth 65 and is rejected by URD204.
    let mut fs = MockFs::new();
    let entry_source = make_source(&["./file_01.urd.md"], "");

    for i in 1..64 {
        let next = format!("./file_{:02}.urd.md", i + 1);
        let src = make_source(&[&next], "");
        let name = format!("file_{:02}.urd.md", i);
        fs = fs.add(&name, &src);
    }
    // file_64 exists but the import from file_63 is rejected at depth 65.
    let src_64 = make_source(&[], "");
    fs = fs.add("file_64.urd.md", &src_64);

    let ast = parse_source("entry.urd.md", &entry_source);
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(find_diagnostic(&diag, "URD204").is_some());
    // entry + file_01 through file_63 = 64 nodes.
    // file_64 is NOT in the graph (depth limit rejected it).
    assert!(!graph.nodes.contains_key("file_64.urd.md"));
    assert_eq!(graph.nodes.len(), 64);
}

#[test]
fn graph_missing_file() {
    let source = make_source(&["./missing.urd.md"], "");
    let ast = parse_source("a.urd.md", &source);
    let fs = MockFs::new(); // No files
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(find_diagnostic(&diag, "URD201").is_some());
    // Missing file absent from graph.
    assert!(!graph.nodes.contains_key("missing.urd.md"));
    assert_eq!(graph.nodes.len(), 1); // Only entry
    let order: Vec<&str> = graph.topological_order().into_iter().map(|s| s.as_str()).collect();
    assert_eq!(order, vec!["a.urd.md"]);
}

#[test]
fn graph_file_too_large() {
    let source = make_source(&["./big.urd.md"], "");
    let ast = parse_source("a.urd.md", &source);
    // 1 MB + 1 byte
    let big_content = "x".repeat(1_048_577);
    let fs = MockFs::new().add("big.urd.md", &big_content);
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(find_diagnostic(&diag, "URD103").is_some());
    assert!(!graph.nodes.contains_key("big.urd.md"));
    assert_eq!(graph.nodes.len(), 1);
}

#[test]
fn graph_stem_collision() {
    // content/tavern.urd.md and scenes/tavern.urd.md share stem "tavern".
    let entry_source = make_source(
        &["./content/tavern.urd.md", "./scenes/tavern.urd.md"],
        "",
    );
    let tavern1 = make_source(&[], "");
    let tavern2 = make_source(&[], "");
    let ast = parse_source("world.urd.md", &entry_source);
    let fs = MockFs::new()
        .add("content/tavern.urd.md", &tavern1)
        .add("scenes/tavern.urd.md", &tavern2);
    let mut diag = DiagnosticCollector::new();

    let _graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(find_diagnostic(&diag, "URD203").is_some());
    let d = find_diagnostic(&diag, "URD203").unwrap();
    assert!(d.message.contains("tavern"), "URD203 message: {}", d.message);
}

// ── Topological sort determinism tests ──────────────────────────────

#[test]
fn topo_linear() {
    let a_source = make_source(&["./b.urd.md"], "");
    let b_source = make_source(&["./c.urd.md"], "");
    let c_source = make_source(&[], "");
    let ast = parse_source("a.urd.md", &a_source);
    let fs = MockFs::new()
        .add("b.urd.md", &b_source)
        .add("c.urd.md", &c_source);
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);
    let order: Vec<&str> = graph.topological_order().into_iter().map(|s| s.as_str()).collect();

    assert_eq!(order, vec!["c.urd.md", "b.urd.md", "a.urd.md"]);
}

#[test]
fn topo_diamond_alphabetical_tie() {
    // A→B, A→C (B and C independent). B before C alphabetically.
    let a_source = make_source(&["./b.urd.md", "./c.urd.md"], "");
    let b_source = make_source(&[], "");
    let c_source = make_source(&[], "");
    let ast = parse_source("a.urd.md", &a_source);
    let fs = MockFs::new()
        .add("b.urd.md", &b_source)
        .add("c.urd.md", &c_source);
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);
    let order: Vec<&str> = graph.topological_order().into_iter().map(|s| s.as_str()).collect();

    assert_eq!(order, vec!["b.urd.md", "c.urd.md", "a.urd.md"]);
}

#[test]
fn topo_wide_fan() {
    // A imports B, C, D, E (all independent).
    let a_source = make_source(
        &[
            "./b.urd.md",
            "./c.urd.md",
            "./d.urd.md",
            "./e.urd.md",
        ],
        "",
    );
    let leaf = make_source(&[], "");
    let ast = parse_source("a.urd.md", &a_source);
    let fs = MockFs::new()
        .add("b.urd.md", &leaf)
        .add("c.urd.md", &leaf)
        .add("d.urd.md", &leaf)
        .add("e.urd.md", &leaf);
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);
    let order: Vec<&str> = graph.topological_order().into_iter().map(|s| s.as_str()).collect();

    assert_eq!(
        order,
        vec!["b.urd.md", "c.urd.md", "d.urd.md", "e.urd.md", "a.urd.md"]
    );
}

#[test]
fn topo_shared_dependency() {
    // A (entry) imports B and C. Both B and C import D.
    let a_source = make_source(&["./b.urd.md", "./c.urd.md"], "");
    let b_source = make_source(&["./d.urd.md"], "");
    let c_source = make_source(&["./d.urd.md"], "");
    let d_source = make_source(&[], "");
    let ast = parse_source("a.urd.md", &a_source);
    let fs = MockFs::new()
        .add("b.urd.md", &b_source)
        .add("c.urd.md", &c_source)
        .add("d.urd.md", &d_source);
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);
    let order: Vec<&str> = graph.topological_order().into_iter().map(|s| s.as_str()).collect();

    // D first (shared dep), B before C (alphabetical), A last (entry).
    assert_eq!(
        order,
        vec!["d.urd.md", "b.urd.md", "c.urd.md", "a.urd.md"]
    );
}

// ── Error recovery tests ────────────────────────────────────────────

#[test]
fn recovery_one_bad_import_among_good() {
    // A imports B (exists) and C (missing).
    let a_source = make_source(&["./b.urd.md", "./c.urd.md"], "");
    let b_source = make_source(&[], "");
    let ast = parse_source("a.urd.md", &a_source);
    let fs = MockFs::new().add("b.urd.md", &b_source);
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(find_diagnostic(&diag, "URD201").is_some());
    assert_eq!(graph.nodes.len(), 2); // A and B
    assert!(!graph.nodes.contains_key("c.urd.md"));
    let order: Vec<&str> = graph.topological_order().into_iter().map(|s| s.as_str()).collect();
    assert_eq!(order, vec!["b.urd.md", "a.urd.md"]);
}

#[test]
fn recovery_cycle_with_other_imports() {
    // A imports B and C. B imports A (cycle) and D.
    let a_source = make_source(&["./b.urd.md", "./c.urd.md"], "");
    let b_source = make_source(&["./a.urd.md", "./d.urd.md"], "");
    let c_source = make_source(&[], "");
    let d_source = make_source(&[], "");
    let ast = parse_source("a.urd.md", &a_source);
    let fs = MockFs::new()
        .add("b.urd.md", &b_source)
        .add("c.urd.md", &c_source)
        .add("d.urd.md", &d_source);
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(find_diagnostic(&diag, "URD202").is_some());
    // All four files in graph (A, B, C, D).
    assert_eq!(graph.nodes.len(), 4);
}

#[test]
fn recovery_parse_failure_in_import() {
    // A imports B. B has unclosed frontmatter → catastrophic parse failure.
    let a_source = make_source(&["./b.urd.md"], "");
    let ast = parse_source("a.urd.md", &a_source);
    // Unclosed frontmatter: only opening ---, no closing ---.
    let b_source = "---\nworld: test\n";
    let fs = MockFs::new().add("b.urd.md", b_source);
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    // B not in graph (catastrophic parse failure).
    assert!(!graph.nodes.contains_key("b.urd.md"));
    assert_eq!(graph.nodes.len(), 1);
    // URD101 from PARSE for B.
    assert!(find_diagnostic(&diag, "URD101").is_some());
}

#[test]
fn recovery_cascading_missing_leaf() {
    // A→B→C, C missing. B and A both in graph.
    let a_source = make_source(&["./b.urd.md"], "");
    let b_source = make_source(&["./c.urd.md"], "");
    let ast = parse_source("a.urd.md", &a_source);
    let fs = MockFs::new().add("b.urd.md", &b_source);
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(find_diagnostic(&diag, "URD201").is_some());
    // B and A in graph. C absent.
    assert_eq!(graph.nodes.len(), 2);
    assert!(graph.nodes.contains_key("a.urd.md"));
    assert!(graph.nodes.contains_key("b.urd.md"));
    assert!(!graph.nodes.contains_key("c.urd.md"));
}

#[test]
fn recovery_invalid_utf8() {
    let a_source = make_source(&["./bad.urd.md"], "");
    let ast = parse_source("a.urd.md", &a_source);
    let fs = MockFs::new().add_error("bad.urd.md", MockFile::InvalidUtf8);
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(find_diagnostic(&diag, "URD212").is_some());
    assert!(!graph.nodes.contains_key("bad.urd.md"));
    assert_eq!(graph.nodes.len(), 1);
}

// ── Filesystem error tests ──────────────────────────────────────────

#[test]
fn fs_permission_denied() {
    let a_source = make_source(&["./secret.urd.md"], "");
    let ast = parse_source("a.urd.md", &a_source);
    let fs = MockFs::new().add_error("secret.urd.md", MockFile::PermissionDenied);
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(find_diagnostic(&diag, "URD213").is_some());
    assert_eq!(graph.nodes.len(), 1);
}

#[test]
fn fs_io_error() {
    let a_source = make_source(&["./broken.urd.md"], "");
    let ast = parse_source("a.urd.md", &a_source);
    let fs = MockFs::new().add_error("broken.urd.md", MockFile::IoError("disk failure".to_string()));
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    let d = find_diagnostic(&diag, "URD214").expect("Expected URD214");
    assert!(d.message.contains("disk failure"));
    assert_eq!(graph.nodes.len(), 1);
}

// ── Span reference tests ───────────────────────────────────────────

#[test]
fn span_urd201_references_import_decl() {
    let source = make_source(&["./missing.urd.md"], "");
    let ast = parse_source("entry.urd.md", &source);
    let fs = MockFs::new();
    let mut diag = DiagnosticCollector::new();

    let _graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    let d = find_diagnostic(&diag, "URD201").expect("Expected URD201");
    // Span should reference the import declaration in entry.urd.md.
    assert_eq!(d.span.file, "entry.urd.md");
    // The import line is line 2 (line 1 is ---, line 2 is import:).
    assert_eq!(d.span.start_line, 2);
    // Message includes source file and line.
    assert!(d.message.contains("entry.urd.md"));
}

#[test]
fn span_urd202_references_cycle_closing_import() {
    let a_source = make_source(&["./b.urd.md"], "");
    let b_source = make_source(&["./a.urd.md"], "");
    let ast = parse_source("a.urd.md", &a_source);
    let fs = MockFs::new().add("b.urd.md", &b_source);
    let mut diag = DiagnosticCollector::new();

    let _graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    let d = find_diagnostic(&diag, "URD202").expect("Expected URD202");
    // Span references the import in B that closes the cycle.
    assert_eq!(d.span.file, "b.urd.md");
    assert_eq!(d.span.start_line, 2);
}

#[test]
fn span_urd207_references_self_import() {
    let source = make_source(&["./world.urd.md"], "");
    let ast = parse_source("world.urd.md", &source);
    let fs = MockFs::new();
    let mut diag = DiagnosticCollector::new();

    let _graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    let d = find_diagnostic(&diag, "URD207").expect("Expected URD207");
    assert_eq!(d.span.file, "world.urd.md");
    assert_eq!(d.span.start_line, 2);
}

// ── Casing mismatch test ────────────────────────────────────────────

#[test]
fn casing_mismatch_warning() {
    // Import with wrong casing, FS reports canonical casing.
    let source = make_source(&["./Types.urd.md"], "");
    let ast = parse_source("world.urd.md", &source);
    let inner_fs = MockFs::new().add("Types.urd.md", &make_source(&[], ""));
    let fs = CasingMockFs::new(inner_fs).add_casing("", "Types.urd.md", "types.urd.md");
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    let d = find_diagnostic(&diag, "URD206").expect("Expected URD206");
    assert!(d.message.contains("Types.urd.md"));
    assert!(d.message.contains("types.urd.md"));
    // File stored under canonical casing.
    assert!(graph.nodes.contains_key("types.urd.md"));
    assert!(!graph.nodes.contains_key("Types.urd.md"));
}

// ── Additional edge cases ───────────────────────────────────────────

#[test]
fn graph_no_frontmatter() {
    // File with no frontmatter — treated as leaf.
    let source = "# Tavern\n\nA bustling tavern.\n";
    let ast = parse_source("world.urd.md", source);
    let fs = MockFs::new();
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(!diag.has_errors());
    assert_eq!(graph.nodes.len(), 1);
}

#[test]
fn graph_empty_file() {
    let ast = parse_source("empty.urd.md", "");
    let fs = MockFs::new();
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(!diag.has_errors());
    assert_eq!(graph.nodes.len(), 1);
}

#[test]
fn graph_entry_dir_prepended_to_fs_path() {
    // When entry_dir is non-empty, it is prepended to normalised paths for FS access.
    let source = make_source(&["./types.urd.md"], "");
    let ast = parse_source("world.urd.md", &source);
    // FS has the file at the full path.
    let fs = MockFs::new().add("content/types.urd.md", &make_source(&[], ""));
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "content/", &mut diag, &fs);

    assert!(!diag.has_errors());
    // Graph stores normalised path (without entry_dir prefix).
    assert!(graph.nodes.contains_key("types.urd.md"));
}

#[test]
fn topo_entry_always_last() {
    // Even when entry has no imports, it appears in the order.
    let source = make_source(&[], "");
    let ast = parse_source("only.urd.md", &source);
    let fs = MockFs::new();
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);
    let order: Vec<&str> = graph.topological_order().into_iter().map(|s| s.as_str()).collect();

    assert_eq!(order, vec!["only.urd.md"]);
}

#[test]
fn multiple_path_validation_errors() {
    // Multiple invalid imports: each produces its own diagnostic.
    let source = make_source(
        &[
            "/absolute.urd.md",
            "./no-ext.txt",
            "./good.urd.md",
        ],
        "",
    );
    let ast = parse_source("world.urd.md", &source);
    let fs = MockFs::new().add("good.urd.md", &make_source(&[], ""));
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(find_diagnostic(&diag, "URD209").is_some());
    assert!(find_diagnostic(&diag, "URD210").is_some());
    // Good import still succeeds.
    assert!(graph.nodes.contains_key("good.urd.md"));
}

// ── Span reference: URD206 ──────────────────────────────────────────

#[test]
fn span_urd206_references_import_decl() {
    // URD206 span should point to the ImportDecl, not the discovered file.
    let source = make_source(&["./Types.urd.md"], "");
    let ast = parse_source("world.urd.md", &source);
    let inner_fs = MockFs::new().add("Types.urd.md", &make_source(&[], ""));
    let fs = CasingMockFs::new(inner_fs).add_casing("", "Types.urd.md", "types.urd.md");
    let mut diag = DiagnosticCollector::new();

    let _graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    let d = find_diagnostic(&diag, "URD206").expect("Expected URD206");
    assert_eq!(d.span.file, "world.urd.md");
    assert_eq!(d.span.start_line, 2); // line 2 is the import declaration
}

// ── Integration tests ───────────────────────────────────────────────

#[test]
fn integration_single_file_no_imports() {
    // Two Room Key Puzzle pattern: single file, realistic content, no imports.
    let source = "\
---
world: two-room-key
  start: cell

types:
  Key [portable]:
    name: string
---

# Cell

A dim stone cell.

[@rusty_key]

* Pick up the rusty key -> @rusty_key
";
    let ast = parse_source("two-room-key.urd.md", source);
    let fs = MockFs::new();
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    // Trivial graph: one file, no edges, zero import diagnostics.
    assert!(!diag.has_errors());
    assert_eq!(count_diagnostics(&diag, "URD201"), 0);
    assert_eq!(count_diagnostics(&diag, "URD202"), 0);
    assert_eq!(graph.nodes.len(), 1);
    assert_eq!(graph.edges.len(), 0);
    let order: Vec<&str> = graph
        .topological_order()
        .into_iter()
        .map(|s| s.as_str())
        .collect();
    assert_eq!(order, vec!["two-room-key.urd.md"]);
}

#[test]
fn integration_two_file_import() {
    // Entry file imports a world definitions file.
    let entry_source = "\
---
import: ./world.urd.md
---

# The Rusty Anchor

A bustling dockside tavern.
";
    let world_source = "\
---
world: tavern
  start: the-rusty-anchor

types:
  NPC [interactable]:
    mood: string = neutral
---
";
    let ast = parse_source("tavern.urd.md", entry_source);
    let fs = MockFs::new().add("world.urd.md", world_source);
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(!diag.has_errors(), "Unexpected errors: {:?}", diag.all());
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
    let order: Vec<&str> = graph
        .topological_order()
        .into_iter()
        .map(|s| s.as_str())
        .collect();
    // world.urd.md first (dependency), tavern.urd.md last (entry).
    assert_eq!(order, vec!["world.urd.md", "tavern.urd.md"]);
}

#[test]
fn integration_multi_file_diamond() {
    // Entry imports types and npcs. npcs imports types (diamond).
    let entry_source = "\
---
import: ./types.urd.md
import: ./npcs.urd.md
---

# The Rusty Anchor

A bustling tavern.
";
    let types_source = "\
---
types:
  NPC [interactable]:
    mood: string = neutral
---
";
    let npcs_source = "\
---
import: ./types.urd.md

entities:
  @arina: NPC { mood: friendly }
---
";
    let ast = parse_source("tavern.urd.md", entry_source);
    let fs = MockFs::new()
        .add("types.urd.md", types_source)
        .add("npcs.urd.md", npcs_source);
    let mut diag = DiagnosticCollector::new();

    let graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    assert!(!diag.has_errors(), "Unexpected errors: {:?}", diag.all());
    assert_eq!(graph.nodes.len(), 3);
    // types.urd.md loaded once despite being imported by both entry and npcs.
    let order: Vec<&str> = graph
        .topological_order()
        .into_iter()
        .map(|s| s.as_str())
        .collect();
    // types first (shared dep), npcs second (depends on types), tavern last (entry).
    assert_eq!(
        order,
        vec!["types.urd.md", "npcs.urd.md", "tavern.urd.md"]
    );
}

// ── Cycle path with subdirectories ──────────────────────────────────

#[test]
fn cycle_path_uses_normalised_paths() {
    // Verify that URD202 cycle path includes full normalised paths, not just filenames.
    let entry_source = make_source(&["./sub/a.urd.md"], "");
    let a_source = make_source(&["./b.urd.md"], "");
    let b_source = make_source(&["../sub/a.urd.md"], "");
    let ast = parse_source("entry.urd.md", &entry_source);
    let fs = MockFs::new()
        .add("sub/a.urd.md", &a_source)
        .add("sub/b.urd.md", &b_source);
    let mut diag = DiagnosticCollector::new();

    let _graph = resolve_imports_with_reader(ast, "", &mut diag, &fs);

    let d = find_diagnostic(&diag, "URD202").expect("Expected URD202");
    // Cycle message should contain full normalised paths including directory.
    assert!(
        d.message.contains("sub/a.urd.md"),
        "Expected 'sub/a.urd.md' in cycle message: {}",
        d.message
    );
}

// ── Helper ──────────────────────────────────────────────────────────

fn graph_is_valid(graph: &urd_compiler::graph::DependencyGraph) -> bool {
    // Basic validity: entry_path set and all edge endpoints exist.
    if let Some(ref entry) = graph.entry_path {
        if !graph.nodes.contains_key(entry) {
            return false;
        }
    }
    for (src, tgt) in &graph.edges {
        if !graph.nodes.contains_key(src) || !graph.nodes.contains_key(tgt) {
            return false;
        }
    }
    true
}
