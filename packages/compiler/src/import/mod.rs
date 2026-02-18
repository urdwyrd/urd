/// Phase 2: IMPORT — entry file AST to dependency graph.
///
/// Input:  Entry `FileAst`
/// Output: `DependencyGraph` + all `FileAst`s in topological order
///
/// Key guarantee: acyclic, depth-limited, file stems unique, paths normalised.
///
/// Diagnostic code range: URD200–URD299
///
/// IMPORT is the only compiler phase that reads from the filesystem.
/// All other phases operate on in-memory data structures.

use std::collections::{BTreeMap, HashSet};

use crate::ast::{FileAst, FrontmatterValue, ImportDecl};
use crate::diagnostics::DiagnosticCollector;
use crate::graph::{file_stem, CompilationUnit, DependencyGraph, FileNode, MAX_FILE_COUNT, MAX_FILE_SIZE, MAX_IMPORT_DEPTH};
use crate::parse;
use crate::span::Span;

// ── Filesystem abstraction ──────────────────────────────────────────

/// Errors that can occur when reading a file from disk.
#[derive(Debug, Clone)]
pub enum FileReadError {
    NotFound,
    PermissionDenied,
    InvalidUtf8,
    IoError(String),
    /// File exceeds the maximum size limit. Contains the file size in bytes.
    TooLarge(usize),
}

/// Abstraction over filesystem operations for testability.
///
/// Production code uses `OsFileReader`. Tests provide an in-memory
/// implementation so that IMPORT can be tested without real I/O.
pub trait FileReader {
    /// Read a file at the given filesystem path, returning its contents.
    fn read_file(&self, fs_path: &str) -> Result<String, FileReadError>;

    /// Discover canonical filename casing on a case-insensitive filesystem.
    ///
    /// Returns `Some(canonical_name)` if the file exists at `dir/filename`
    /// but with different casing than `filename`. Returns `None` if casing
    /// matches, the file does not exist, or the platform provides no way
    /// to detect casing differences.
    fn canonical_filename(&self, dir: &str, filename: &str) -> Option<String>;
}

/// Production filesystem reader using OS APIs.
pub struct OsFileReader;

impl FileReader for OsFileReader {
    fn read_file(&self, fs_path: &str) -> Result<String, FileReadError> {
        // Check file size via metadata before reading to avoid loading
        // oversized files into memory.
        match std::fs::metadata(fs_path) {
            Ok(meta) => {
                let size = meta.len() as usize;
                if size > MAX_FILE_SIZE {
                    return Err(FileReadError::TooLarge(size));
                }
            }
            Err(e) => {
                return match e.kind() {
                    std::io::ErrorKind::NotFound => Err(FileReadError::NotFound),
                    std::io::ErrorKind::PermissionDenied => Err(FileReadError::PermissionDenied),
                    _ => Err(FileReadError::IoError(e.to_string())),
                };
            }
        }

        match std::fs::read(fs_path) {
            Ok(bytes) => String::from_utf8(bytes).map_err(|_| FileReadError::InvalidUtf8),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Err(FileReadError::NotFound),
                std::io::ErrorKind::PermissionDenied => Err(FileReadError::PermissionDenied),
                _ => Err(FileReadError::IoError(e.to_string())),
            },
        }
    }

    fn canonical_filename(&self, dir: &str, filename: &str) -> Option<String> {
        // Case-insensitive filesystems (Windows, macOS): enumerate directory
        // to discover actual casing.
        #[cfg(any(target_os = "windows", target_os = "macos"))]
        {
            let dir_path = if dir.is_empty() { "." } else { dir };
            if let Ok(entries) = std::fs::read_dir(dir_path) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.eq_ignore_ascii_case(filename) && name != filename {
                            return Some(name.to_string());
                        }
                    }
                }
            }
            None
        }
        // Case-sensitive filesystems: casing mismatch means file not found.
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            let _ = (dir, filename);
            None
        }
    }
}

// ── Path utilities ──────────────────────────────────────────────────

/// Extract the directory part of a path. Returns `""` if no directory.
fn path_dir(path: &str) -> &str {
    match path.rfind('/') {
        Some(pos) => &path[..pos + 1],
        None => "",
    }
}

/// Extract the filename part of a path (after the last `/`).
fn path_filename(path: &str) -> &str {
    match path.rfind('/') {
        Some(pos) => &path[pos + 1..],
        None => path,
    }
}

/// Collapse `.` and `..` segments in a path.
/// Returns `None` if the path would escape above the project root.
fn collapse_dotdot(path: &str) -> Option<String> {
    let mut segments: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        if segment == ".." {
            if segments.is_empty() {
                return None; // Would go above project root
            }
            segments.pop();
        } else if segment != "." && !segment.is_empty() {
            segments.push(segment);
        }
    }
    Some(segments.join("/"))
}

// ── Path validation ─────────────────────────────────────────────────

/// Validate an import path after trimming and backslash normalisation.
/// Checks run on `written_path`, before filesystem access.
/// Returns `true` if the path passes all pre-validation checks.
fn validate_import_path(
    written_path: &str,
    source_file: &str,
    import_span: &Span,
    diagnostics: &mut DiagnosticCollector,
) -> bool {
    // URD211: Empty path
    if written_path.is_empty() {
        diagnostics.error(
            "URD211",
            format!("Empty import path at {}:{}.", source_file, import_span.start_line),
            import_span.clone(),
        );
        return false;
    }

    // URD209: Absolute path (Unix `/` or Windows drive letter `C:`)
    if written_path.starts_with('/')
        || (written_path.len() >= 2
            && written_path.as_bytes()[0].is_ascii_alphabetic()
            && written_path.as_bytes()[1] == b':')
    {
        diagnostics.error(
            "URD209",
            format!(
                "Absolute import paths are not supported: '{}'.",
                written_path
            ),
            import_span.clone(),
        );
        return false;
    }

    // URD210: Missing .urd.md extension
    if !written_path.ends_with(".urd.md") {
        diagnostics.error(
            "URD210",
            format!(
                "Import path '{}' does not have the .urd.md extension.",
                written_path
            ),
            import_span.clone(),
        );
        return false;
    }

    true
}

/// Resolve an import path relative to the importing file.
/// Returns the normalised path (relative to entry directory).
/// Emits URD208 and returns `None` if the path escapes the project root.
fn resolve_import_path(
    written_path: &str,
    importer_path: &str,
    import_span: &Span,
    diagnostics: &mut DiagnosticCollector,
) -> Option<String> {
    // Strip leading ./
    let stripped = written_path.strip_prefix("./").unwrap_or(written_path);

    // Get importer's directory (relative to entry dir)
    let dir = path_dir(importer_path);

    // Join with importer's directory
    let joined = if dir.is_empty() {
        stripped.to_string()
    } else {
        format!("{}{}", dir, stripped)
    };

    // Collapse .. segments
    match collapse_dotdot(&joined) {
        Some(normalised) => Some(normalised),
        None => {
            diagnostics.error(
                "URD208",
                format!(
                    "Import path '{}' resolves outside the project root.",
                    written_path
                ),
                import_span.clone(),
            );
            None
        }
    }
}

// ── Public entry points ─────────────────────────────────────────────

/// Resolve all imports starting from the entry file AST.
///
/// Discovers imported files recursively, parses each via PARSE, builds
/// the dependency graph, and produces a topologically sorted file list.
///
/// The `entry_dir` is the directory containing the entry file, used to
/// construct filesystem paths for reading imported files. It may be empty
/// if the entry file is in the current working directory.
pub fn resolve_imports(
    entry_ast: FileAst,
    entry_dir: &str,
    diagnostics: &mut DiagnosticCollector,
) -> CompilationUnit {
    resolve_imports_with_reader(entry_ast, entry_dir, diagnostics, &OsFileReader)
}

/// Resolve all imports with an explicit file reader (for testing).
pub fn resolve_imports_with_reader(
    entry_ast: FileAst,
    entry_dir: &str,
    diagnostics: &mut DiagnosticCollector,
    reader: &dyn FileReader,
) -> CompilationUnit {
    let mut graph = DependencyGraph::new();

    let entry_path = entry_ast.path.clone();
    let entry_imports = extract_import_decls(&entry_ast);

    // Add entry file to graph.
    graph.nodes.insert(
        entry_path.clone(),
        FileNode {
            path: entry_path.clone(),
            ast: entry_ast,
            imports: Vec::new(),
        },
    );
    graph.entry_path = Some(entry_path.clone());

    // Traversal state.
    let mut visited: HashSet<String> = HashSet::new();
    visited.insert(entry_path.clone());
    let mut traversal_stack: Vec<String> = vec![entry_path.clone()];

    // Process entry file's imports.
    process_imports(
        &entry_path,
        &entry_imports,
        entry_dir,
        &mut graph,
        &mut visited,
        &mut traversal_stack,
        diagnostics,
        reader,
    );

    // Post-discovery checks.
    check_file_count(&graph, diagnostics);
    check_file_stems(&graph, diagnostics);

    // Build ordered_asts from topological order.
    let ordered_asts = graph.topological_order().into_iter().cloned().collect();

    CompilationUnit {
        graph,
        ordered_asts,
    }
}

// ── Import extraction ───────────────────────────────────────────────

/// Extract `ImportDecl` nodes from a `FileAst`'s frontmatter.
fn extract_import_decls(ast: &FileAst) -> Vec<ImportDecl> {
    let Some(ref fm) = ast.frontmatter else {
        return Vec::new();
    };

    let mut imports = Vec::new();
    for entry in &fm.entries {
        if let FrontmatterValue::ImportDecl(ref decl) = entry.value {
            imports.push(decl.clone());
        }
    }
    imports
}

// ── Recursive discovery ─────────────────────────────────────────────

/// Process all import declarations for a single file.
fn process_imports(
    importer_path: &str,
    import_decls: &[ImportDecl],
    entry_dir: &str,
    graph: &mut DependencyGraph,
    visited: &mut HashSet<String>,
    traversal_stack: &mut Vec<String>,
    diagnostics: &mut DiagnosticCollector,
    reader: &dyn FileReader,
) {
    // Track edges from this file to prevent duplicate edges.
    let mut edges_from_this_file: HashSet<String> = HashSet::new();

    for decl in import_decls {
        process_single_import(
            importer_path,
            decl,
            entry_dir,
            graph,
            visited,
            traversal_stack,
            &mut edges_from_this_file,
            diagnostics,
            reader,
        );
    }
}

/// Process a single `ImportDecl` — the core of the discovery algorithm.
///
/// Steps a–j from the IMPORT brief's Recursive Discovery section.
#[allow(clippy::too_many_arguments)]
fn process_single_import(
    importer_path: &str,
    decl: &ImportDecl,
    entry_dir: &str,
    graph: &mut DependencyGraph,
    visited: &mut HashSet<String>,
    traversal_stack: &mut Vec<String>,
    edges_from_this_file: &mut HashSet<String>,
    diagnostics: &mut DiagnosticCollector,
    reader: &dyn FileReader,
) {
    // Step a: Trim and validate the path.
    let trimmed = decl.path.trim();
    let written_path = trimmed.replace('\\', "/");

    if !validate_import_path(&written_path, importer_path, &decl.span, diagnostics) {
        return;
    }

    // Step b: Resolve the path.
    let mut normalised_path =
        match resolve_import_path(&written_path, importer_path, &decl.span, diagnostics) {
            Some(p) => p,
            None => return,
        };

    // Step c: Check for self-import.
    if normalised_path == importer_path {
        diagnostics.error(
            "URD207",
            format!("File imports itself: '{}'.", written_path),
            decl.span.clone(),
        );
        return;
    }

    // Step d: Check for cycles.
    // The cycle path uses normalised_path values for consistency with graph identity.
    if let Some(cycle_start) = traversal_stack.iter().position(|p| p == &normalised_path) {
        let cycle_path: Vec<&str> = traversal_stack[cycle_start..]
            .iter()
            .map(|s| s.as_str())
            .chain(std::iter::once(normalised_path.as_str()))
            .collect();
        let cycle_display = cycle_path.join(" \u{2192} ");
        diagnostics.error(
            "URD202",
            format!("Circular import detected: {}.", cycle_display),
            decl.span.clone(),
        );
        return;
    }

    // Step e: Check import depth.
    if traversal_stack.len() >= MAX_IMPORT_DEPTH {
        diagnostics.error(
            "URD204",
            "Import depth limit exceeded (64 files in chain).",
            decl.span.clone(),
        );
        return;
    }

    // Step f: Check for already-loaded file.
    if visited.contains(&normalised_path) {
        add_edge(importer_path, &normalised_path, edges_from_this_file, graph);
        return;
    }

    // Step g: Load the new file.
    let fs_path = if entry_dir.is_empty() {
        normalised_path.clone()
    } else {
        format!("{}{}", entry_dir, normalised_path)
    };

    let source = match reader.read_file(&fs_path) {
        Ok(s) => {
            // Casing mismatch detection (step g, after locating file).
            let filename = path_filename(&normalised_path);
            let fs_dir = if entry_dir.is_empty() {
                path_dir(&normalised_path).to_string()
            } else {
                format!("{}{}", entry_dir, path_dir(&normalised_path))
            };

            if let Some(canonical) = reader.canonical_filename(&fs_dir, filename) {
                let dir_part = path_dir(&normalised_path);
                let corrected = if dir_part.is_empty() {
                    canonical.clone()
                } else {
                    format!("{}{}", dir_part, canonical)
                };

                diagnostics.warning(
                    "URD206",
                    format!(
                        "Import path '{}' differs in filename casing from discovered file '{}'. Using discovered casing.",
                        written_path, corrected
                    ),
                    decl.span.clone(),
                );

                normalised_path = corrected;

                // Re-check visited set with canonical path.
                if visited.contains(&normalised_path) {
                    add_edge(importer_path, &normalised_path, edges_from_this_file, graph);
                    return;
                }
            }

            s
        }
        Err(FileReadError::NotFound) => {
            diagnostics.error(
                "URD201",
                format!(
                    "Imported file not found: '{}' (imported from {}:{}).",
                    written_path, importer_path, decl.span.start_line
                ),
                decl.span.clone(),
            );
            return;
        }
        Err(FileReadError::PermissionDenied) => {
            diagnostics.error(
                "URD213",
                format!(
                    "Cannot read file '{}': permission denied.",
                    normalised_path
                ),
                decl.span.clone(),
            );
            return;
        }
        Err(FileReadError::InvalidUtf8) => {
            diagnostics.error(
                "URD212",
                format!("File contains invalid UTF-8: '{}'.", normalised_path),
                decl.span.clone(),
            );
            return;
        }
        Err(FileReadError::IoError(msg)) => {
            diagnostics.error(
                "URD214",
                format!("I/O error reading '{}': {}.", normalised_path, msg),
                decl.span.clone(),
            );
            return;
        }
        Err(FileReadError::TooLarge(size)) => {
            diagnostics.error(
                "URD103",
                format!(
                    "File exceeds 1 MB size limit: {} is {} bytes.",
                    normalised_path, size
                ),
                decl.span.clone(),
            );
            return;
        }
    };

    // Check file size (URD103) — belt-and-braces for readers that skip metadata.
    if source.len() > MAX_FILE_SIZE {
        diagnostics.error(
            "URD103",
            format!(
                "File exceeds 1 MB size limit: {} is {} bytes.",
                normalised_path,
                source.len()
            ),
            decl.span.clone(),
        );
        return;
    }

    // Step h: Parse the file.
    let file_ast = match parse::parse(&normalised_path, &source, diagnostics) {
        Some(ast) => ast,
        None => return, // Catastrophic parse failure
    };

    // Step i: Add to graph.
    let new_imports = extract_import_decls(&file_ast);
    graph.nodes.insert(
        normalised_path.clone(),
        FileNode {
            path: normalised_path.clone(),
            ast: file_ast,
            imports: Vec::new(),
        },
    );
    visited.insert(normalised_path.clone());
    add_edge(importer_path, &normalised_path, edges_from_this_file, graph);

    // Step j: Recurse.
    traversal_stack.push(normalised_path.clone());
    process_imports(
        &normalised_path,
        &new_imports,
        entry_dir,
        graph,
        visited,
        traversal_stack,
        diagnostics,
        reader,
    );
    traversal_stack.pop();
}

/// Add a dependency edge from `importer` to `target`, deduplicating.
/// Also updates the importer's `imports` list.
fn add_edge(
    importer: &str,
    target: &str,
    edges_from_this_file: &mut HashSet<String>,
    graph: &mut DependencyGraph,
) {
    if edges_from_this_file.insert(target.to_string()) {
        graph.edges.push((importer.to_string(), target.to_string()));
        if let Some(node) = graph.nodes.get_mut(importer) {
            if !node.imports.contains(&target.to_string()) {
                node.imports.push(target.to_string());
            }
        }
    }
}

// ── Post-discovery checks ───────────────────────────────────────────

/// URD205: Check that the compilation unit does not exceed 256 files.
fn check_file_count(graph: &DependencyGraph, diagnostics: &mut DiagnosticCollector) {
    let count = graph.nodes.len();
    if count > MAX_FILE_COUNT {
        diagnostics.error(
            "URD205",
            format!(
                "Compilation unit exceeds 256 files ({} discovered).",
                count
            ),
            Span::synthetic(),
        );
    }
}

/// URD203: Check that all file stems are unique.
fn check_file_stems(graph: &DependencyGraph, diagnostics: &mut DiagnosticCollector) {
    // Build a map from stem to sorted list of paths.
    let mut stems: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for path in graph.nodes.keys() {
        let stem = file_stem(path);
        stems.entry(stem).or_default().push(path);
    }

    for (stem, mut paths) in stems {
        if paths.len() > 1 {
            paths.sort();
            // Report all pairs.
            for i in 0..paths.len() {
                for j in (i + 1)..paths.len() {
                    diagnostics.error(
                        "URD203",
                        format!(
                            "File stem collision: '{}' is produced by both {} and {}. Rename one file to avoid section ID conflicts.",
                            stem, paths[i], paths[j]
                        ),
                        Span::synthetic(),
                    );
                }
            }
        }
    }
}
