use urd_grammar::parse;

// ═══════════════════════════════════════════════════════════════
// POSITIVE CORPUS — these files must parse successfully
// ═══════════════════════════════════════════════════════════════

fn assert_valid(path: &str) {
    let input = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path, e));
    match parse(&input) {
        Ok(_) => {}
        Err(e) => panic!("Expected {} to parse successfully, but got error:\n{}", path, e),
    }
}

#[test]
fn valid_tavern() {
    assert_valid("tests/valid/tavern.urd.md");
}

#[test]
fn valid_monty_hall() {
    assert_valid("tests/valid/monty-hall.urd.md");
}

#[test]
fn valid_key_puzzle() {
    assert_valid("tests/valid/key-puzzle.urd.md");
}

#[test]
fn valid_interrogation() {
    assert_valid("tests/valid/interrogation.urd.md");
}

#[test]
fn valid_edge_cases_frontmatter() {
    assert_valid("tests/valid/edge-cases-frontmatter.urd.md");
}

#[test]
fn valid_edge_cases_comments() {
    assert_valid("tests/valid/edge-cases-comments.urd.md");
}

#[test]
fn valid_edge_cases_structure() {
    assert_valid("tests/valid/edge-cases-structure.urd.md");
}

// ═══════════════════════════════════════════════════════════════
// NEGATIVE CORPUS — these files must fail to parse
// ═══════════════════════════════════════════════════════════════

fn assert_invalid(path: &str, expected_line: usize, expected_col: usize) {
    let input = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path, e));
    match parse(&input) {
        Ok(_) => panic!("Expected {} to fail parsing, but it succeeded", path),
        Err(e) => {
            // Extract line and column from pest error
            let (line, col) = match e.line_col {
                pest::error::LineColLocation::Pos((l, c)) => (l, c),
                pest::error::LineColLocation::Span((l, c), _) => (l, c),
            };
            assert_eq!(
                line, expected_line,
                "{}: expected error on line {}, got line {}.\nFull error:\n{}",
                path, expected_line, line, e
            );
            assert_eq!(
                col, expected_col,
                "{}: expected error at column {}, got column {}.\nFull error:\n{}",
                path, expected_col, col, e
            );
        }
    }
}

#[test]
fn invalid_tabs() {
    // Pest reports error at line 1 col 1 ("expected Frontmatter") because
    // the tab causes FrontmatterBody to fail, Frontmatter fails, and the
    // second File alternative (!("---" ~ NEWLINE)) also fails.
    assert_invalid("tests/invalid/bad-tabs.urd.md", 1, 1);
}

#[test]
fn invalid_unclosed_frontmatter() {
    // Opening '---' without closing '---'. Error at EOF or at the
    // point where the parser realises frontmatter is unclosed.
    let path = "tests/invalid/bad-unclosed-frontmatter.urd.md";
    let input = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path, e));
    match parse(&input) {
        Ok(_) => panic!("Expected {} to fail parsing, but it succeeded", path),
        Err(_) => {
            // Parse failure is sufficient — exact position depends on
            // how far the parser gets before failing.
        }
    }
}

#[test]
fn invalid_malformed_entity() {
    // @Guard-1 — uppercase 'G' violates Identifier rule (lowercase start).
    // Line 7 (after frontmatter + blank + heading + blank), col 2 (the 'G' after '@').
    assert_invalid("tests/invalid/bad-malformed-entity.urd.md", 7, 2);
}

#[test]
fn invalid_empty_choice() {
    // '* ' followed by newline — Text requires at least one character.
    // Line 7, col 3 (after '* ', where Text is expected).
    assert_invalid("tests/invalid/bad-empty-choice.urd.md", 7, 3);
}

#[test]
fn invalid_empty_heading() {
    // '# ' followed by newline — Text requires at least one character.
    // Line 1, col 3 (after '# ', where Text is expected).
    assert_invalid("tests/invalid/bad-empty-heading.urd.md", 1, 3);
}
