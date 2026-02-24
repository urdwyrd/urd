/// FactSet-derived diagnostics (ANALYZE phase).
///
/// Every function in this module operates solely on the FactSet and
/// PropertyDependencyIndex — no AST, no symbol table, no source text.
///
/// ## Code Range
///
/// | Phase              | Range         |
/// |--------------------|---------------|
/// | ANALYZE (FactSet)  | URD600–URD699 |

use crate::diagnostics::{Diagnostic, RelatedInfo, Severity};
use crate::facts::{
    CompareOp, FactSet, LiteralKind, PropertyDependencyIndex, WriteOp,
};

/// Run all FactSet-derived diagnostics.
///
/// Called after `extract_facts()`, before or alongside VALIDATE.
pub fn analyze(fact_set: &FactSet) -> Vec<Diagnostic> {
    let index = PropertyDependencyIndex::build(fact_set);
    let mut diagnostics = Vec::new();

    diagnostics.extend(check_read_never_written(fact_set, &index));
    diagnostics.extend(check_written_never_read(fact_set, &index));
    diagnostics.extend(check_enum_variant_untested(fact_set, &index));
    diagnostics.extend(check_unreachable_threshold(fact_set, &index));
    diagnostics.extend(check_circular_dependency(fact_set, &index));

    diagnostics
}

/// D1: Property read but never written — URD601
///
/// A property appears in conditions but no effect anywhere modifies it.
fn check_read_never_written(
    fact_set: &FactSet,
    index: &PropertyDependencyIndex,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for key in index.read_properties() {
        if !index.writes_of(key).is_empty() {
            continue;
        }

        let read_indices = index.reads_of(key);
        if read_indices.is_empty() {
            continue;
        }

        let first = &fact_set.reads()[read_indices[0]];
        let related: Vec<RelatedInfo> = read_indices[1..]
            .iter()
            .map(|&i| {
                let r = &fact_set.reads()[i];
                RelatedInfo {
                    message: format!(
                        "Also read at {}:{}",
                        r.span.file, r.span.start_line
                    ),
                    span: r.span.clone(),
                }
            })
            .collect();

        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: "URD601".to_string(),
            message: format!(
                "Property '{}.{}' is read in conditions but never written by any effect. \
                 It will always reflect its default or initial value.",
                key.entity_type, key.property
            ),
            span: first.span.clone(),
            suggestion: None,
            related,
        });
    }

    diagnostics
}

/// D2: Property written but never read — URD602
///
/// A property appears in effects but no condition anywhere tests it.
fn check_written_never_read(
    fact_set: &FactSet,
    index: &PropertyDependencyIndex,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for key in index.written_properties() {
        if !index.reads_of(key).is_empty() {
            continue;
        }

        let write_indices = index.writes_of(key);
        if write_indices.is_empty() {
            continue;
        }

        let first = &fact_set.writes()[write_indices[0]];
        let related: Vec<RelatedInfo> = write_indices[1..]
            .iter()
            .map(|&i| {
                let w = &fact_set.writes()[i];
                RelatedInfo {
                    message: format!(
                        "Also written at {}:{}",
                        w.span.file, w.span.start_line
                    ),
                    span: w.span.clone(),
                }
            })
            .collect();

        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: "URD602".to_string(),
            message: format!(
                "Property '{}.{}' is written by effects but never read in any condition. \
                 The writes have no observable effect on game logic.",
                key.entity_type, key.property
            ),
            span: first.span.clone(),
            suggestion: None,
            related,
        });
    }

    diagnostics
}

/// D3: Effect produces enum variant unreachable by any condition — URD603
///
/// An effect writes an enum property to a specific variant, but no condition
/// in the entire world ever tests for that variant.
///
/// Skips any PropertyKey for which D2 would fire (property is never read at all),
/// since that is redundant noise.
fn check_enum_variant_untested(
    fact_set: &FactSet,
    index: &PropertyDependencyIndex,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for write in fact_set.writes() {
        // Only interested in `Set` writes with an identifier value (enum variant).
        if write.operator != WriteOp::Set {
            continue;
        }
        let value_kind = match &write.value_kind {
            Some(kind) => kind,
            None => continue,
        };
        if *value_kind != LiteralKind::Ident {
            continue;
        }

        let key = write.key();

        // Skip if property is never read at all (D2 already covers this).
        if index.reads_of(&key).is_empty() {
            continue;
        }

        let written_variant = &write.value_expr;

        // Collect all variants tested via `== variant` comparisons.
        let tested_variants: std::collections::HashSet<&str> = index
            .reads_of(&key)
            .iter()
            .filter_map(|&i| {
                let r = &fact_set.reads()[i];
                if r.operator == CompareOp::Eq {
                    Some(r.value_literal.as_str())
                } else {
                    None
                }
            })
            .collect();

        if !tested_variants.contains(written_variant.as_str()) {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                code: "URD603".to_string(),
                message: format!(
                    "Effect sets '{}.{}' to '{}' but no condition anywhere tests for this variant. \
                     The write may have no observable effect.",
                    key.entity_type, key.property, written_variant
                ),
                span: write.span.clone(),
                suggestion: None,
                related: vec![],
            });
        }
    }

    diagnostics
}

/// D4: Condition tests unreachable threshold — URD604
///
/// A condition compares a numeric property against a threshold that no
/// combination of effects in the world can reach (conservatively: skips if
/// any Add/Sub writes exist, since accumulation bounds are unknown).
fn check_unreachable_threshold(
    fact_set: &FactSet,
    index: &PropertyDependencyIndex,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for read in fact_set.reads() {
        // Only interested in numeric comparisons with ordering operators.
        let is_ordering = matches!(
            read.operator,
            CompareOp::Lt | CompareOp::Gt | CompareOp::Le | CompareOp::Ge
        );
        if !is_ordering {
            continue;
        }
        if read.value_kind != LiteralKind::Int {
            continue;
        }

        let threshold: i64 = match read.value_literal.parse() {
            Ok(v) => v,
            Err(_) => continue,
        };

        let key = read.key();
        let write_indices = index.writes_of(&key);

        // Check if any Add/Sub writes exist — conservative skip.
        let has_add_sub = write_indices.iter().any(|&i| {
            let w = &fact_set.writes()[i];
            w.operator == WriteOp::Add || w.operator == WriteOp::Sub
        });
        if has_add_sub {
            continue;
        }

        // Only Set writes remain (or no writes at all). Check if any satisfies the threshold.
        let any_satisfies = write_indices.iter().any(|&i| {
            let w = &fact_set.writes()[i];
            if w.operator != WriteOp::Set {
                return false;
            }
            let value_kind = match &w.value_kind {
                Some(kind) => kind,
                None => return false,
            };
            if *value_kind != LiteralKind::Int {
                return false;
            }
            let value: i64 = match w.value_expr.parse() {
                Ok(v) => v,
                Err(_) => return false,
            };
            satisfies_comparison(value, &read.operator, threshold)
        });

        if !any_satisfies {
            let op_str = match read.operator {
                CompareOp::Lt => "<",
                CompareOp::Gt => ">",
                CompareOp::Le => "<=",
                CompareOp::Ge => ">=",
                _ => unreachable!(),
            };
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                code: "URD604".to_string(),
                message: format!(
                    "Condition compares '{}.{}' against {} but no effect can produce a value \
                     that satisfies '{} {}'. The condition may never be true.",
                    key.entity_type, key.property, threshold, op_str, threshold
                ),
                span: read.span.clone(),
                suggestion: None,
                related: vec![],
            });
        }
    }

    diagnostics
}

/// D5: Circular property dependency — URD605
///
/// Every effect that writes a property is guarded by a condition that reads
/// that same property. Without an unguarded write path or a satisfying
/// initial value, the property can never change.
fn check_circular_dependency(
    fact_set: &FactSet,
    index: &PropertyDependencyIndex,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for key in index.written_properties() {
        let write_indices = index.writes_of(key);
        if write_indices.is_empty() {
            continue;
        }

        let mut guarded_count = 0usize;
        let mut related = Vec::new();

        for &write_idx in write_indices {
            let w = &fact_set.writes()[write_idx];
            let site_read_indices = fact_set.read_indices_for_site(&w.site);

            let self_read = site_read_indices
                .iter()
                .find(|&&i| fact_set.reads()[i].key() == *key);

            if let Some(&read_idx) = self_read {
                guarded_count += 1;
                let r = &fact_set.reads()[read_idx];
                related.push(RelatedInfo {
                    message: format!(
                        "Write at {}:{} is guarded by condition reading '{}.{}' at {}:{}",
                        w.span.file,
                        w.span.start_line,
                        key.entity_type,
                        key.property,
                        r.span.file,
                        r.span.start_line
                    ),
                    span: w.span.clone(),
                });
            }
        }

        if guarded_count == write_indices.len() && guarded_count > 0 {
            let first = &fact_set.writes()[write_indices[0]];
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                code: "URD605".to_string(),
                message: format!(
                    "Property '{}.{}' may be stuck in a circular dependency. \
                     Every effect that writes this property is guarded by a condition \
                     that reads it. Without an unguarded write path or a satisfying \
                     initial value, the property can never change.",
                    key.entity_type, key.property
                ),
                span: first.span.clone(),
                suggestion: None,
                related,
            });
        }
    }

    diagnostics
}

/// Returns true if `value` satisfies the comparison `value <op> threshold`.
fn satisfies_comparison(value: i64, op: &CompareOp, threshold: i64) -> bool {
    match op {
        CompareOp::Lt => value < threshold,
        CompareOp::Gt => value > threshold,
        CompareOp::Le => value <= threshold,
        CompareOp::Ge => value >= threshold,
        CompareOp::Eq => value == threshold,
        CompareOp::Ne => value != threshold,
    }
}
