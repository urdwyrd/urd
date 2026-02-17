/// Tests for Phase 1: PARSE
///
/// Test categories from the PARSE phase brief:
/// - Every grammar rule exercised
/// - Canonical integration files parsed with zero errors
/// - Error recovery scenarios
/// - Span accuracy verification

use urd_compiler::parse;
use urd_compiler::diagnostics::DiagnosticCollector;

#[test]
#[should_panic(expected = "not yet implemented")]
fn parse_stub_is_unimplemented() {
    let mut diag = DiagnosticCollector::new();
    parse::parse(&"test.urd.md".to_string(), "---\nworld: test\n---\n", &mut diag);
}

// Placeholder test cases — to be expanded per the PARSE phase brief:
//
// Frontmatter parsing:
//   - Empty frontmatter
//   - World block
//   - Type definitions with traits and properties
//   - Entity declarations with overrides
//   - Import declarations
//
// Content parsing:
//   - Location headings
//   - Section labels
//   - Entity presence lists
//   - Entity speech
//   - Stage directions
//   - Prose
//   - Choices (sticky and non-sticky, with targets)
//   - Conditions (property comparison, containment, exhaustion)
//   - Effects (set, move, reveal, destroy)
//   - Jumps (plain and exit-qualified)
//   - Exit declarations with conditions and blocked messages
//   - Rule blocks with select clauses
//   - Comments
//
// Error recovery:
//   - Unclosed frontmatter
//   - Unrecognised syntax in content
//   - Tab characters
//   - Malformed conditions
//   - Malformed effects
//
// Span tracking:
//   - Verify spans are accurate for all node types
