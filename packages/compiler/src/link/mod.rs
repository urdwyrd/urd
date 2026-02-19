/// Phase 3: LINK — declaration collection, reference resolution, ID derivation.
///
/// Input:  `CompilationUnit` (from IMPORT)
/// Output: `LinkedWorld` (symbol table + annotated ASTs + graph)
///
/// Two sequential passes over all `FileAst`s in topological order:
///   Pass 1 (collection): register every declaration in the symbol table.
///   Pass 2 (resolution): resolve every reference, fill annotation slots.
///
/// Key guarantees: every declared name registered, every resolvable
/// reference annotated, duplicates flagged, visible scope enforced.
///
/// Diagnostic code range: URD300–URD399

pub mod collect;
pub mod resolve;

use std::collections::BTreeSet;

use indexmap::IndexMap;

use crate::ast::Scalar;
use crate::diagnostics::DiagnosticCollector;
use crate::graph::{CompilationUnit, DependencyGraph};
use crate::span::Span;
use crate::symbol_table::{PropertyType, SymbolTable, Value};

/// The output of the LINK phase: annotated ASTs + populated symbol table.
#[derive(Debug)]
pub struct LinkedWorld {
    pub graph: DependencyGraph,
    pub symbol_table: SymbolTable,
}

/// Per-file metadata computed during collection, used during resolution.
#[derive(Debug)]
pub(crate) struct FileContext {
    pub file_stem: String,
    pub visible_scope: BTreeSet<String>,
    /// local section name → compiled section ID (for jump resolution within a file).
    pub local_sections: IndexMap<String, String>,
}

/// Stored `world.start` / `world.entry` for resolution in pass 2.
#[derive(Debug, Default)]
pub(crate) struct WorldConfig {
    pub start: Option<(String, Span)>,
    pub entry: Option<(String, Span)>,
}

/// Result of a scope-checked symbol lookup.
pub(crate) enum ResolveResult<'a, V> {
    Found(&'a V),
    NotVisible { declared_in_file: String },
    NotFound,
}

/// Top-level LINK entry point.
///
/// Consumes the `CompilationUnit` from IMPORT. Creates the symbol table
/// internally. Returns `LinkedWorld` with populated symbol table and
/// annotated ASTs.
pub fn link(
    mut compilation_unit: CompilationUnit,
    diagnostics: &mut DiagnosticCollector,
) -> LinkedWorld {
    let mut symbol_table = SymbolTable::default();
    let mut world_config = WorldConfig::default();
    let mut file_contexts: IndexMap<String, FileContext> = IndexMap::new();

    // Pass 1: Collection
    collect::collect(
        &compilation_unit.graph,
        &compilation_unit.ordered_asts,
        &mut symbol_table,
        &mut world_config,
        &mut file_contexts,
        diagnostics,
    );

    // Pass 2: Resolution
    resolve::resolve(
        &mut compilation_unit.graph,
        &compilation_unit.ordered_asts,
        &mut symbol_table,
        &world_config,
        &file_contexts,
        diagnostics,
    );

    LinkedWorld {
        graph: compilation_unit.graph,
        symbol_table,
    }
}

// ── Shared helpers ──

/// Compute the visible scope for a file: {self} ∪ {direct imports}.
pub(crate) fn visible_scope(file_path: &str, graph: &DependencyGraph) -> BTreeSet<String> {
    let mut scope = BTreeSet::new();
    scope.insert(file_path.to_string());
    if let Some(node) = graph.nodes.get(file_path) {
        for imp in &node.imports {
            scope.insert(imp.clone());
        }
    }
    scope
}

/// Scope-checked lookup in a symbol namespace.
///
/// Returns `Found` if the name exists and its declaring file is in the visible scope.
/// Returns `NotVisible` if the name exists but is out of scope.
/// Returns `NotFound` if the name does not exist in the namespace.
pub(crate) fn resolve_in_scope<'a, V>(
    name: &str,
    namespace: &'a IndexMap<String, V>,
    declared_in_file: impl Fn(&V) -> &str,
    visible_scope: &BTreeSet<String>,
) -> ResolveResult<'a, V> {
    match namespace.get(name) {
        Some(symbol) => {
            let file = declared_in_file(symbol);
            if visible_scope.contains(file) {
                ResolveResult::Found(symbol)
            } else {
                ResolveResult::NotVisible {
                    declared_in_file: file.to_string(),
                }
            }
        }
        None => ResolveResult::NotFound,
    }
}

/// Compute Levenshtein edit distance between two strings.
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut prev: Vec<usize> = (0..=b_len).collect();
    let mut curr = vec![0; b_len + 1];

    for (i, a_ch) in a.chars().enumerate() {
        curr[0] = i + 1;
        for (j, b_ch) in b.chars().enumerate() {
            let cost = if a_ch == b_ch { 0 } else { 1 };
            curr[j + 1] = (prev[j] + cost)
                .min(prev[j + 1] + 1)
                .min(curr[j] + 1);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b_len]
}

/// Find the best suggestion from a namespace for a misspelled name.
/// Returns the first candidate with edit distance ≤ 2, preferring smallest distance,
/// then insertion order (deterministic via IndexMap).
pub(crate) fn find_suggestion<V>(name: &str, namespace: &IndexMap<String, V>) -> Option<String> {
    let mut best: Option<(String, usize)> = None;
    for key in namespace.keys() {
        let dist = edit_distance(name, key);
        if dist > 0 && dist <= 2 {
            if best.as_ref().map_or(true, |(_, d)| dist < *d) {
                best = Some((key.clone(), dist));
            }
        }
    }
    best.map(|(name, _)| name)
}

/// Parse a property type string into a `PropertyType` enum.
pub(crate) fn parse_property_type(s: &str) -> PropertyType {
    match s {
        "bool" | "boolean" => PropertyType::Boolean,
        "int" | "integer" => PropertyType::Integer,
        "num" | "number" => PropertyType::Number,
        "str" | "string" => PropertyType::String,
        "enum" => PropertyType::Enum,
        "ref" => PropertyType::Ref,
        "list" => PropertyType::List,
        _ => PropertyType::String,
    }
}

/// Convert an AST `Scalar` to a symbol table `Value`.
pub(crate) fn scalar_to_value(s: &Scalar) -> Value {
    match s {
        Scalar::String(v) => Value::String(v.clone()),
        Scalar::Integer(v) => Value::Integer(*v),
        Scalar::Number(v) => Value::Number(*v),
        Scalar::Boolean(v) => Value::Boolean(*v),
    }
}
