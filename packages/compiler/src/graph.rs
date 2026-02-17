/// The dependency graph: file import structure.
///
/// IMPORT builds this directed acyclic graph of file imports. It is used
/// during compilation for ordering and cycle detection.
///
/// Properties (enforced by IMPORT):
/// - Acyclic: cycles are rejected with a diagnostic.
/// - Depth-limited: max 64 levels of import chaining.
/// - Non-transitive: A imports B imports C does NOT give A access to C.
/// - Stable: same source files produce the same graph.

use std::collections::HashMap;

use crate::ast::FileAst;
use crate::span::FilePath;

/// A node in the dependency graph: one parsed file.
#[derive(Debug)]
pub struct FileNode {
    pub path: FilePath,
    pub ast: FileAst,
    pub imports: Vec<FilePath>,
}

/// The dependency graph produced by the IMPORT phase.
#[derive(Debug, Default)]
pub struct DependencyGraph {
    pub nodes: HashMap<FilePath, FileNode>,
    pub edges: Vec<(FilePath, FilePath)>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns files in topological order (dependencies first, entry file last).
    /// Ties at the same depth are broken alphabetically by normalised path.
    pub fn topological_order(&self) -> Vec<&FilePath> {
        // Stub: will be implemented in the IMPORT phase.
        // For now, return all node paths sorted alphabetically.
        let mut paths: Vec<&FilePath> = self.nodes.keys().collect();
        paths.sort();
        paths
    }

    /// Returns the file stems for all nodes.
    pub fn file_stems(&self) -> HashMap<String, FilePath> {
        let mut stems = HashMap::new();
        for path in self.nodes.keys() {
            let stem = file_stem(path);
            stems.insert(stem, path.clone());
        }
        stems
    }
}

/// Extract the file stem from a path: strip directory and `.urd.md` extension.
/// `content/tavern.urd.md` → `tavern`
pub fn file_stem(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    name.strip_suffix(".urd.md")
        .unwrap_or(name)
        .to_string()
}

/// Maximum import chain depth.
pub const MAX_IMPORT_DEPTH: usize = 64;

/// Maximum files in a compilation unit.
pub const MAX_FILE_COUNT: usize = 256;

/// Maximum file size in bytes (1 MB).
pub const MAX_FILE_SIZE: usize = 1_048_576;

/// Maximum choice nesting depth (error at 4, warn at 3).
pub const MAX_CHOICE_NESTING_DEPTH: usize = 4;

/// Choice nesting depth that triggers a warning.
pub const WARN_CHOICE_NESTING_DEPTH: usize = 3;

/// Maximum frontmatter nesting depth.
pub const MAX_FRONTMATTER_NESTING_DEPTH: usize = 8;
