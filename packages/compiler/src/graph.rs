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

use std::collections::BTreeSet;

use indexmap::IndexMap;

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
    pub nodes: IndexMap<FilePath, FileNode>,
    pub edges: Vec<(FilePath, FilePath)>,
    /// The entry file's normalised path. Set by IMPORT.
    pub entry_path: Option<FilePath>,
}

/// The output of the IMPORT phase: dependency graph + topologically sorted ASTs.
///
/// IMPORT always returns a `CompilationUnit`. Even if every import fails,
/// the entry file is still a valid single-file compilation unit.
#[derive(Debug)]
pub struct CompilationUnit {
    pub graph: DependencyGraph,
    /// Every successfully parsed `FileAst` in topological order.
    /// Dependencies before dependents, entry file always last.
    /// Ties broken alphabetically by normalised path.
    pub ordered_asts: Vec<FilePath>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns files in topological order (dependencies first, entry file last).
    /// Ties at the same depth are broken alphabetically by normalised path.
    ///
    /// Uses Kahn's algorithm with a BTreeSet for deterministic ordering.
    /// The entry file is excluded from the priority queue and appended last.
    pub fn topological_order(&self) -> Vec<&FilePath> {
        let entry = match &self.entry_path {
            Some(p) => p,
            None => {
                // No entry path set — return alphabetically sorted.
                let mut paths: Vec<&FilePath> = self.nodes.keys().collect();
                paths.sort();
                return paths;
            }
        };

        if self.nodes.len() <= 1 {
            return self.nodes.keys().collect();
        }

        // dep_count[N] = number of files N imports that are present in the graph.
        let mut dep_count: IndexMap<&FilePath, usize> = IndexMap::new();
        // reverse_adj[M] = list of files that import M.
        let mut reverse_adj: IndexMap<&FilePath, Vec<&FilePath>> = IndexMap::new();

        for (path, node) in &self.nodes {
            let count = node.imports.iter()
                .filter(|imp| self.nodes.contains_key(imp.as_str()))
                .count();
            dep_count.insert(path, count);

            for imp in &node.imports {
                if self.nodes.contains_key(imp.as_str()) {
                    reverse_adj.entry(imp).or_default().push(path);
                }
            }
        }

        // Initial ready set: nodes with 0 dependencies, excluding entry.
        let mut ready: BTreeSet<&FilePath> = BTreeSet::new();
        for (&path, &count) in &dep_count {
            if count == 0 && path != entry {
                ready.insert(path);
            }
        }

        let mut result = Vec::with_capacity(self.nodes.len());

        while let Some(&path) = ready.iter().next() {
            ready.remove(path);
            result.push(path);

            if let Some(dependents) = reverse_adj.get(path) {
                for &dep in dependents {
                    if dep == entry { continue; }
                    if let Some(count) = dep_count.get_mut(dep) {
                        *count -= 1;
                        if *count == 0 {
                            ready.insert(dep);
                        }
                    }
                }
            }
        }

        // Entry file always last.
        result.push(entry);
        result
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
