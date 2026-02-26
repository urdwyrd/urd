/// Tests for SymbolTable and DependencyGraph fields on CompilationResult.
///
/// Verifies that symbol_table and graph are present whenever LINK succeeds
/// (including validation-error paths) and absent when compilation fails
/// before LINK (PARSE failure, IMPORT failure).

use urd_compiler::compile;

fn fixture_path(name: &str) -> String {
    let base = env!("CARGO_MANIFEST_DIR");
    format!("{}/tests/fixtures/{}", base, name)
}

// ── Success path ──

#[test]
fn result_has_symbol_table_on_success() {
    let result = compile(&fixture_path("locked-garden.urd.md"));
    assert!(result.success);
    assert!(result.symbol_table.is_some(), "symbol_table should be Some on success");
    assert!(result.graph.is_some(), "graph should be Some on success");
}

// ── Validation error path (LINK succeeds, VALIDATE emits errors) ──

#[test]
fn result_has_symbol_table_on_validate_error() {
    let result = compile(&fixture_path("negative-unresolved-entity.urd.md"));
    assert!(!result.success);
    assert!(result.world.is_none());
    assert!(result.symbol_table.is_some(), "symbol_table should be Some when LINK succeeds");
    assert!(result.graph.is_some(), "graph should be Some when LINK succeeds");
}

// ── PARSE failure path ──

#[test]
fn result_no_symbol_table_on_parse_error() {
    // Unclosed frontmatter triggers URD101 and parse returns None.
    let result = urd_compiler::compile_source("broken.urd.md", "---\nworld:\n  name: broken\n");
    assert!(!result.success);
    assert!(result.symbol_table.is_none(), "symbol_table should be None on PARSE failure");
    assert!(result.graph.is_none(), "graph should be None on PARSE failure");
}

// ── IMPORT failure path ──

#[test]
fn result_no_symbol_table_on_import_error() {
    let result = compile(&fixture_path("negative-missing-import.urd.md"));
    assert!(!result.success);
    assert!(result.symbol_table.is_none(), "symbol_table should be None on IMPORT failure");
    assert!(result.graph.is_none(), "graph should be None on IMPORT failure");
}

// ── SymbolTable consistency with DefinitionIndex ──

#[test]
fn result_symbol_table_matches_definition_index() {
    let result = compile(&fixture_path("locked-garden.urd.md"));
    let st = result.symbol_table.as_ref().expect("symbol_table present");
    let di = result.definition_index.as_ref().expect("definition_index present");

    // Every type in the symbol table has a corresponding definition index entry.
    for type_name in st.types.keys() {
        let key = format!("type:{}", type_name);
        assert!(
            di.get(&key).is_some(),
            "DefinitionIndex missing entry for type '{}'",
            type_name
        );
    }

    // Every entity in the symbol table has a corresponding definition index entry.
    for entity_id in st.entities.keys() {
        let key = format!("entity:@{}", entity_id);
        assert!(
            di.get(&key).is_some(),
            "DefinitionIndex missing entry for entity '@{}'",
            entity_id
        );
    }
}

// ── DependencyGraph structure ──

#[test]
fn result_graph_has_entry_path() {
    let result = compile(&fixture_path("locked-garden.urd.md"));
    let graph = result.graph.as_ref().expect("graph present");
    assert!(
        graph.entry_path.is_some(),
        "graph should have an entry_path"
    );
}

#[test]
fn result_graph_nodes_have_asts() {
    let result = compile(&fixture_path("interrogation/main.urd.md"));
    let graph = result.graph.as_ref().expect("graph present");
    // Multi-file fixture: should have entries for both files.
    assert!(
        graph.nodes.len() >= 2,
        "graph should have at least 2 file nodes, got {}",
        graph.nodes.len()
    );
    for (path, node) in &graph.nodes {
        // Each node should have a parsed AST with a valid span.
        assert!(
            node.ast.span.start_line >= 1,
            "AST for '{}' should have a valid span",
            path
        );
        // The entry file (main.urd.md) should have body content.
        if path.contains("main") {
            assert!(
                !node.ast.content.is_empty(),
                "Entry file '{}' should have non-empty content",
                path
            );
        }
    }
}

#[test]
fn result_graph_edges_match_imports() {
    let result = compile(&fixture_path("interrogation/main.urd.md"));
    let graph = result.graph.as_ref().expect("graph present");
    // The main file imports world.urd.md — exactly one edge.
    assert!(
        !graph.edges.is_empty(),
        "graph should have at least one import edge"
    );
    // Count total imports declared across all files.
    let total_imports: usize = graph
        .nodes
        .values()
        .map(|node| node.imports.len())
        .sum();
    assert_eq!(
        graph.edges.len(),
        total_imports,
        "edge count should match total import count"
    );
}
