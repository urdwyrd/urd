/// Pass 2: Resolution — walk every AST again, resolve references, fill annotations.
///
/// For each file in topological order:
/// 1. Load the file's `FileContext` (visible scope, local sections).
/// 2. Resolve frontmatter references (entity types, property overrides, world config).
/// 3. Resolve content references (entity refs, property accesses, jumps, exits, etc.).

use std::collections::BTreeSet;

use indexmap::IndexMap;

use crate::ast::{
    Annotation, ConditionExpr, ContainerKind, ContentNode, DestinationKind, EffectType,
    FrontmatterValue,
};
use crate::diagnostics::{DiagnosticCollector, Diagnostic, Severity};
use crate::graph::DependencyGraph;
use crate::slugify::slugify;
use crate::span::{FilePath, Span};
use crate::symbol_table::SymbolTable;

use super::{find_suggestion, resolve_in_scope, FileContext, ResolveResult, WorldConfig};

/// Run resolution pass over all files.
pub(crate) fn resolve(
    graph: &mut DependencyGraph,
    ordered_asts: &[FilePath],
    symbol_table: &mut SymbolTable,
    world_config: &WorldConfig,
    file_contexts: &IndexMap<String, FileContext>,
    diagnostics: &mut DiagnosticCollector,
) {
    // Resolve world.start and world.entry — store results for VALIDATE.
    if let Some((start_val, _span)) = &world_config.start {
        let slug = slugify(start_val);
        if symbol_table.locations.contains_key(&slug) {
            symbol_table.world_start = Some(slug);
        }
    }
    if let Some((entry_val, _span)) = &world_config.entry {
        if symbol_table.sequences.contains_key(entry_val) {
            symbol_table.world_entry = Some(entry_val.clone());
        }
    }

    for file_path in ordered_asts {
        let ctx = match file_contexts.get(file_path.as_str()) {
            Some(c) => c,
            None => continue,
        };

        let node = match graph.nodes.get_mut(file_path.as_str()) {
            Some(n) => n,
            None => continue,
        };

        // Resolve frontmatter.
        if let Some(fm) = &mut node.ast.frontmatter {
            for entry in &mut fm.entries {
                resolve_frontmatter_entry(
                    &mut entry.value,
                    file_path,
                    &ctx.visible_scope,
                    symbol_table,
                    diagnostics,
                );
            }
        }

        // Resolve content — track location context.
        let mut current_location_id: Option<String> = None;

        for content in &mut node.ast.content {
            resolve_content_node(
                content,
                file_path,
                ctx,
                &mut current_location_id,
                symbol_table,
                diagnostics,
            );
        }
    }
}

/// Resolve references in a frontmatter entry.
fn resolve_frontmatter_entry(
    value: &mut FrontmatterValue,
    file_path: &str,
    visible_scope: &BTreeSet<String>,
    symbol_table: &mut SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    match value {
        FrontmatterValue::EntityDecl(ed) => {
            // Resolve type_name → TypeSymbol.
            let type_resolved = match resolve_in_scope(
                &ed.type_name,
                &symbol_table.types,
                |ts| ts.declared_in.file.as_str(),
                visible_scope,
            ) {
                ResolveResult::Found(ts) => {
                    // Store the type name in the EntitySymbol.
                    if let Some(entity_sym) = symbol_table.entities.get_mut(&ed.id) {
                        entity_sym.type_symbol = Some(ts.name.clone());
                    }
                    // Annotate the EntityDecl AST node.
                    ed.annotation = Some(Annotation {
                        resolved_entity: Some(ed.id.clone()),
                        resolved_type: Some(ts.name.clone()),
                        ..Default::default()
                    });
                    true
                }
                ResolveResult::NotVisible { declared_in_file } => {
                    diagnostics.emit(Diagnostic {
                        severity: Severity::Error,
                        code: "URD301".to_string(),
                        message: format!(
                            "Unresolved type reference '{}'.",
                            ed.type_name,
                        ),
                        span: ed.span.clone(),
                        suggestion: Some(format!(
                            "'{}' is declared in {} but {} is not imported by {}.",
                            ed.type_name, declared_in_file, declared_in_file, file_path,
                        )),
                        related: Vec::new(),
                    });
                    false
                }
                ResolveResult::NotFound => {
                    let mut diag = Diagnostic {
                        severity: Severity::Error,
                        code: "URD307".to_string(),
                        message: format!(
                            "Unknown type '{}' for entity '@{}'.",
                            ed.type_name, ed.id,
                        ),
                        span: ed.span.clone(),
                        suggestion: None,
                        related: Vec::new(),
                    };
                    if let Some(suggestion) = find_suggestion(&ed.type_name, &symbol_table.types) {
                        diag.suggestion = Some(format!("Did you mean '{}'?", suggestion));
                    }
                    diagnostics.emit(diag);
                    false
                }
            };

            // Resolve property overrides — only if type resolved.
            if type_resolved {
                let type_name = ed.type_name.clone();
                let overrides: Vec<String> = ed.property_overrides.iter().map(|(k, _)| k.clone()).collect();
                for prop_name in &overrides {
                    if let Some(ts) = symbol_table.types.get(&type_name) {
                        if !ts.properties.contains_key(prop_name)
                            && !IMPLICIT_PROPERTIES.contains(&prop_name.as_str())
                        {
                            diagnostics.error(
                                "URD308",
                                format!(
                                    "Property '{}' does not exist on type '{}'.",
                                    prop_name, type_name,
                                ),
                                ed.span.clone(),
                            );
                        }
                    }
                }
            }
        }

        // Map values (from `types:` / `entities:` blocks) — recurse into entries.
        FrontmatterValue::Map(entries) => {
            for entry in entries {
                resolve_frontmatter_entry(
                    &mut entry.value,
                    file_path,
                    visible_scope,
                    symbol_table,
                    diagnostics,
                );
            }
        }

        _ => {}
    }
}

/// Resolve references in a content node (recursive for nested content).
fn resolve_content_node(
    node: &mut ContentNode,
    file_path: &str,
    ctx: &FileContext,
    current_location_id: &mut Option<String>,
    symbol_table: &mut SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    match node {
        ContentNode::LocationHeading(loc) => {
            let id = slugify(&loc.display_name);
            if !id.is_empty() {
                *current_location_id = Some(id);
            }
        }

        ContentNode::EntitySpeech(speech) => {
            resolve_entity_ref(
                &speech.entity_ref,
                &mut speech.annotation,
                &speech.span,
                file_path,
                &ctx.visible_scope,
                symbol_table,
                diagnostics,
            );
        }

        ContentNode::StageDirection(sd) => {
            resolve_entity_ref(
                &sd.entity_ref,
                &mut sd.annotation,
                &sd.span,
                file_path,
                &ctx.visible_scope,
                symbol_table,
                diagnostics,
            );
        }

        ContentNode::EntityPresence(ep) => {
            if current_location_id.is_none() {
                // URD314 already emitted during collection.
                return;
            }
            let loc_id = current_location_id.as_ref().unwrap().clone();
            for (i, entity_ref) in ep.entity_refs.iter().enumerate() {
                let mut annotation = ep.annotations.get(i).cloned().flatten();
                let resolved = resolve_entity_ref_value(
                    entity_ref,
                    &ep.span,
                    file_path,
                    &ctx.visible_scope,
                    symbol_table,
                    diagnostics,
                );
                if let Some(entity_id) = &resolved {
                    annotation = Some(Annotation {
                        resolved_entity: Some(entity_id.clone()),
                        ..Default::default()
                    });
                    // Add to LocationSymbol.contains.
                    if let Some(loc_sym) = symbol_table.locations.get_mut(&loc_id) {
                        if !loc_sym.contains.contains(entity_id) {
                            loc_sym.contains.push(entity_id.clone());
                        }
                    }
                }
                if i < ep.annotations.len() {
                    ep.annotations[i] = annotation;
                }
            }
        }

        ContentNode::Choice(choice) => {
            // Resolve entity target if present.
            if let Some(target) = &choice.target {
                let resolved = resolve_entity_ref_value(
                    target,
                    &choice.span,
                    file_path,
                    &ctx.visible_scope,
                    symbol_table,
                    diagnostics,
                );
                if let Some(entity_id) = resolved {
                    choice.annotation = Some(Annotation {
                        resolved_entity: Some(entity_id),
                        ..Default::default()
                    });
                }
            }

            // Recurse into choice content.
            for child in &mut choice.content {
                resolve_content_node(
                    child,
                    file_path,
                    ctx,
                    current_location_id,
                    symbol_table,
                    diagnostics,
                );
            }
        }

        ContentNode::Condition(cond) => {
            resolve_condition_expr(
                &mut cond.expr,
                file_path,
                ctx,
                symbol_table,
                diagnostics,
            );
        }

        ContentNode::OrConditionBlock(or_block) => {
            for expr in &mut or_block.conditions {
                resolve_condition_expr(
                    expr,
                    file_path,
                    ctx,
                    symbol_table,
                    diagnostics,
                );
            }
        }

        ContentNode::Effect(effect) => {
            resolve_effect(
                &mut effect.effect_type,
                &mut effect.annotation,
                &effect.span,
                file_path,
                ctx,
                symbol_table,
                diagnostics,
            );
        }

        ContentNode::Jump(jump) => {
            resolve_jump(
                jump,
                file_path,
                ctx,
                current_location_id,
                symbol_table,
                diagnostics,
            );
        }

        ContentNode::ExitDeclaration(exit) => {
            // Resolve exit destination.
            if current_location_id.is_none() {
                // URD314 already emitted during collection.
                return;
            }
            let dest_slug = slugify(&exit.destination);
            if !dest_slug.is_empty() {
                match resolve_in_scope(
                    &dest_slug,
                    &symbol_table.locations,
                    |ls| ls.declared_in.file.as_str(),
                    &ctx.visible_scope,
                ) {
                    ResolveResult::Found(_) => {
                        let loc_id = current_location_id.as_ref().unwrap();
                        if let Some(loc_sym) = symbol_table.locations.get_mut(loc_id) {
                            if let Some(exit_sym) = loc_sym.exits.get_mut(&exit.direction) {
                                exit_sym.resolved_destination = Some(dest_slug.clone());
                            }
                        }
                        exit.annotation = Some(Annotation {
                            resolved_location: Some(dest_slug),
                            ..Default::default()
                        });
                    }
                    ResolveResult::NotVisible { declared_in_file } => {
                        diagnostics.emit(Diagnostic {
                            severity: Severity::Error,
                            code: "URD312".to_string(),
                            message: format!(
                                "Exit destination '{}' does not resolve to any known location.",
                                exit.destination,
                            ),
                            span: exit.span.clone(),
                            suggestion: Some(format!(
                                "'{}' is declared in {} but {} is not imported by {}.",
                                exit.destination, declared_in_file, declared_in_file, file_path,
                            )),
                            related: Vec::new(),
                        });
                    }
                    ResolveResult::NotFound => {
                        diagnostics.error(
                            "URD312",
                            format!(
                                "Exit destination '{}' does not resolve to any known location.",
                                exit.destination,
                            ),
                            exit.span.clone(),
                        );
                    }
                }
            }

            // Resolve references within exit children.
            for child in &mut exit.children {
                resolve_content_node(
                    child,
                    file_path,
                    ctx,
                    current_location_id,
                    symbol_table,
                    diagnostics,
                );
            }
        }

        ContentNode::RuleBlock(rule) => {
            // Resolve entity refs in select clause.
            if let Some(select) = &rule.select {
                for entity_ref in &select.entity_refs {
                    resolve_entity_ref_value(
                        entity_ref,
                        &rule.span,
                        file_path,
                        &ctx.visible_scope,
                        symbol_table,
                        diagnostics,
                    );
                }
                for expr in &mut rule.select.as_mut().unwrap().where_clauses {
                    resolve_condition_expr(expr, file_path, ctx, symbol_table, diagnostics);
                }
            }
            // Resolve entity refs in rule where_clauses.
            for expr in &mut rule.where_clauses {
                resolve_condition_expr(expr, file_path, ctx, symbol_table, diagnostics);
            }
            // Resolve effects.
            for effect in &mut rule.effects {
                resolve_effect(
                    &mut effect.effect_type,
                    &mut effect.annotation,
                    &effect.span,
                    file_path,
                    ctx,
                    symbol_table,
                    diagnostics,
                );
            }
        }

        ContentNode::BlockedMessage(_) | ContentNode::Prose(_) | ContentNode::Comment(_) => {
            // No references to resolve.
        }

        ContentNode::SectionLabel(_) | ContentNode::SequenceHeading(_) | ContentNode::PhaseHeading(_) => {
            // Already processed during collection.
        }

        ContentNode::ErrorNode(_) => {
            // Skip silently.
        }
    }
}

/// Resolve an entity reference and populate the annotation slot.
fn resolve_entity_ref(
    entity_ref: &str,
    annotation: &mut Option<Annotation>,
    span: &Span,
    file_path: &str,
    visible_scope: &BTreeSet<String>,
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    if let Some(entity_id) = resolve_entity_ref_value(
        entity_ref,
        span,
        file_path,
        visible_scope,
        symbol_table,
        diagnostics,
    ) {
        *annotation = Some(Annotation {
            resolved_entity: Some(entity_id),
            ..Default::default()
        });
    }
}

/// Resolve an entity reference string, returning the resolved entity ID or None.
fn resolve_entity_ref_value(
    entity_ref: &str,
    span: &Span,
    file_path: &str,
    visible_scope: &BTreeSet<String>,
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) -> Option<String> {
    match resolve_in_scope(
        entity_ref,
        &symbol_table.entities,
        |es| es.declared_in.file.as_str(),
        visible_scope,
    ) {
        ResolveResult::Found(es) => Some(es.id.clone()),
        ResolveResult::NotVisible { declared_in_file } => {
            diagnostics.emit(Diagnostic {
                severity: Severity::Error,
                code: "URD301".to_string(),
                message: format!("Unresolved entity reference '@{}'.", entity_ref),
                span: span.clone(),
                suggestion: Some(format!(
                    "'@{}' is declared in {} but {} is not imported by {}.",
                    entity_ref, declared_in_file, declared_in_file, file_path,
                )),
                related: Vec::new(),
            });
            None
        }
        ResolveResult::NotFound => {
            let mut diag = Diagnostic {
                severity: Severity::Error,
                code: "URD301".to_string(),
                message: format!("Unresolved entity reference '@{}'.", entity_ref),
                span: span.clone(),
                suggestion: None,
                related: Vec::new(),
            };
            if let Some(suggestion) = find_suggestion(entity_ref, &symbol_table.entities) {
                diag.suggestion = Some(format!("Did you mean '@{}'?", suggestion));
            }
            diagnostics.emit(diag);
            None
        }
    }
}

/// Internal enum used by resolve_container_or_destination before mapping to
/// ContainerKind or DestinationKind.
enum ContainerOrDest {
    KeywordPlayer,
    KeywordHere,
    EntityRef(String),
    LocationRef(String),
}

/// Resolve a container or destination reference using keyword-first priority:
/// 1. "player" → KeywordPlayer
/// 2. "here" → KeywordHere
/// 3. Entity lookup (scope-checked)
/// 4. Location lookup (scope-checked, slugified)
/// 5. Not found → emit URD301, return None
fn resolve_container_or_destination(
    ref_token: &str,
    span: &Span,
    file_path: &str,
    visible_scope: &BTreeSet<String>,
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) -> Option<ContainerOrDest> {
    // Keywords take priority over everything.
    if ref_token == "player" {
        return Some(ContainerOrDest::KeywordPlayer);
    }
    if ref_token == "here" {
        return Some(ContainerOrDest::KeywordHere);
    }

    // Try entity lookup (scope-checked).
    match resolve_in_scope(
        ref_token,
        &symbol_table.entities,
        |es| es.declared_in.file.as_str(),
        visible_scope,
    ) {
        ResolveResult::Found(es) => {
            return Some(ContainerOrDest::EntityRef(es.id.clone()));
        }
        ResolveResult::NotVisible { declared_in_file } => {
            diagnostics.emit(Diagnostic {
                severity: Severity::Error,
                code: "URD301".to_string(),
                message: format!("Unresolved reference '{}'.", ref_token),
                span: span.clone(),
                suggestion: Some(format!(
                    "'{}' is declared in {} but {} is not imported by {}.",
                    ref_token, declared_in_file, declared_in_file, file_path,
                )),
                related: Vec::new(),
            });
            return None;
        }
        ResolveResult::NotFound => {
            // Fall through to location lookup.
        }
    }

    // Try location lookup (scope-checked, slugified).
    let slug = slugify(ref_token);
    if !slug.is_empty() {
        match resolve_in_scope(
            &slug,
            &symbol_table.locations,
            |ls| ls.declared_in.file.as_str(),
            visible_scope,
        ) {
            ResolveResult::Found(_) => {
                return Some(ContainerOrDest::LocationRef(slug));
            }
            ResolveResult::NotVisible { declared_in_file } => {
                diagnostics.emit(Diagnostic {
                    severity: Severity::Error,
                    code: "URD301".to_string(),
                    message: format!("Unresolved reference '{}'.", ref_token),
                    span: span.clone(),
                    suggestion: Some(format!(
                        "'{}' is declared in {} but {} is not imported by {}.",
                        ref_token, declared_in_file, declared_in_file, file_path,
                    )),
                    related: Vec::new(),
                });
                return None;
            }
            ResolveResult::NotFound => {
                // Fall through to error.
            }
        }
    }

    // Not found anywhere.
    let mut diag = Diagnostic {
        severity: Severity::Error,
        code: "URD301".to_string(),
        message: format!("Unresolved reference '{}'.", ref_token),
        span: span.clone(),
        suggestion: None,
        related: Vec::new(),
    };
    // Try edit distance suggestion against both entities and locations.
    if let Some(suggestion) = find_suggestion(ref_token, &symbol_table.entities) {
        diag.suggestion = Some(format!("Did you mean '@{}'?", suggestion));
    } else if let Some(suggestion) = find_suggestion(&slug, &symbol_table.locations) {
        diag.suggestion = Some(format!("Did you mean '{}'?", suggestion));
    }
    diagnostics.emit(diag);
    None
}

/// Resolve a condition expression's entity/property/section references.
fn resolve_condition_expr(
    expr: &mut ConditionExpr,
    file_path: &str,
    ctx: &FileContext,
    symbol_table: &mut SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    match expr {
        ConditionExpr::PropertyComparison(pc) => {
            // Reserved bindings ("target", "player") resolve at runtime, not statically.
            // Skip entity lookup — these are not entity references.
            if pc.entity_ref == "target" || pc.entity_ref == "player" {
                pc.annotation = Some(Annotation {
                    resolved_entity: Some(pc.entity_ref.clone()),
                    ..Default::default()
                });
                return;
            }

            let entity_resolved = resolve_entity_ref_value(
                &pc.entity_ref,
                &pc.span,
                file_path,
                &ctx.visible_scope,
                symbol_table,
                diagnostics,
            );

            if let Some(entity_id) = &entity_resolved {
                pc.annotation = Some(Annotation {
                    resolved_entity: Some(entity_id.clone()),
                    ..Default::default()
                });

                // Resolve property access — only if entity resolved.
                if let Some(es) = symbol_table.entities.get(entity_id) {
                    if let Some(type_name) = &es.type_symbol {
                        if let Some(ts) = symbol_table.types.get(type_name) {
                            if ts.properties.contains_key(&pc.property)
                                || IMPLICIT_PROPERTIES.contains(&pc.property.as_str())
                            {
                                if let Some(ann) = &mut pc.annotation {
                                    ann.resolved_property = Some(pc.property.clone());
                                    ann.resolved_type = Some(type_name.clone());
                                }
                            } else {
                                diagnostics.error(
                                    "URD308",
                                    format!(
                                        "Property '{}' does not exist on type '{}'.",
                                        pc.property, type_name,
                                    ),
                                    pc.span.clone(),
                                );
                            }
                        }
                    }
                }
            }
        }

        ConditionExpr::ContainmentCheck(cc) => {
            let entity_resolved = resolve_entity_ref_value(
                &cc.entity_ref,
                &cc.span,
                file_path,
                &ctx.visible_scope,
                symbol_table,
                diagnostics,
            );

            // Resolve container_ref: keywords first, then entity, then location.
            let container_kind = resolve_container_or_destination(
                &cc.container_ref,
                &cc.span,
                file_path,
                &ctx.visible_scope,
                symbol_table,
                diagnostics,
            );

            if entity_resolved.is_some() || container_kind.is_some() {
                cc.annotation = Some(Annotation {
                    resolved_entity: entity_resolved,
                    container_kind: container_kind.map(|k| match k {
                        ContainerOrDest::KeywordPlayer => ContainerKind::KeywordPlayer,
                        ContainerOrDest::KeywordHere => ContainerKind::KeywordHere,
                        ContainerOrDest::EntityRef(id) => ContainerKind::EntityRef(id),
                        ContainerOrDest::LocationRef(id) => ContainerKind::LocationRef(id),
                    }),
                    ..Default::default()
                });
            }
        }

        ConditionExpr::ExhaustionCheck(ec) => {
            // Resolve section name to a section in the current file.
            if let Some(compiled_id) = ctx.local_sections.get(&ec.section_name) {
                ec.annotation = Some(Annotation {
                    resolved_section: Some(compiled_id.clone()),
                    ..Default::default()
                });
            } else {
                diagnostics.error(
                    "URD309",
                    format!(
                        "Unresolved section '{}' in exhaustion check. No section with this name exists in the current file.",
                        ec.section_name,
                    ),
                    ec.span.clone(),
                );
            }
        }
    }
}

/// Resolve an effect's entity/property references.
fn resolve_effect(
    effect_type: &mut EffectType,
    annotation: &mut Option<Annotation>,
    span: &Span,
    file_path: &str,
    ctx: &FileContext,
    symbol_table: &mut SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    match effect_type {
        EffectType::Set { target_prop, .. } | EffectType::Reveal { target_prop } => {
            // target_prop format: "@entity.property"
            if let Some(stripped) = target_prop.strip_prefix('@') {
                if let Some(dot_pos) = stripped.find('.') {
                    let entity_ref = &stripped[..dot_pos];
                    let property = &stripped[dot_pos + 1..];

                    let entity_resolved = resolve_entity_ref_value(
                        entity_ref,
                        span,
                        file_path,
                        &ctx.visible_scope,
                        symbol_table,
                        diagnostics,
                    );

                    if let Some(entity_id) = &entity_resolved {
                        let mut ann = Annotation {
                            resolved_entity: Some(entity_id.clone()),
                            ..Default::default()
                        };

                        // Resolve property — no cascading if entity type unresolved.
                        if let Some(es) = symbol_table.entities.get(entity_id) {
                            if let Some(type_name) = &es.type_symbol {
                                if let Some(ts) = symbol_table.types.get(type_name) {
                                    if ts.properties.contains_key(property)
                                        || IMPLICIT_PROPERTIES.contains(&property)
                                    {
                                        ann.resolved_property = Some(property.to_string());
                                        ann.resolved_type = Some(type_name.clone());
                                    } else {
                                        diagnostics.error(
                                            "URD308",
                                            format!(
                                                "Property '{}' does not exist on type '{}'.",
                                                property, type_name,
                                            ),
                                            span.clone(),
                                        );
                                    }
                                }
                            }
                        }

                        *annotation = Some(ann);
                    }
                }
            }
        }

        EffectType::Move { entity_ref, destination_ref } => {
            let entity_resolved = resolve_entity_ref_value(
                entity_ref,
                span,
                file_path,
                &ctx.visible_scope,
                symbol_table,
                diagnostics,
            );

            // Resolve destination_ref: keywords first, then entity, then location.
            let dest_kind = resolve_container_or_destination(
                destination_ref,
                span,
                file_path,
                &ctx.visible_scope,
                symbol_table,
                diagnostics,
            );

            if entity_resolved.is_some() || dest_kind.is_some() {
                *annotation = Some(Annotation {
                    resolved_entity: entity_resolved,
                    destination_kind: dest_kind.map(|k| match k {
                        ContainerOrDest::KeywordPlayer => DestinationKind::KeywordPlayer,
                        ContainerOrDest::KeywordHere => DestinationKind::KeywordHere,
                        ContainerOrDest::EntityRef(id) => DestinationKind::EntityRef(id),
                        ContainerOrDest::LocationRef(id) => DestinationKind::LocationRef(id),
                    }),
                    ..Default::default()
                });
            }
        }

        EffectType::Destroy { entity_ref } => {
            let entity_resolved = resolve_entity_ref_value(
                entity_ref,
                span,
                file_path,
                &ctx.visible_scope,
                symbol_table,
                diagnostics,
            );

            if let Some(entity_id) = entity_resolved {
                *annotation = Some(Annotation {
                    resolved_entity: Some(entity_id),
                    ..Default::default()
                });
            }
        }
    }
}

/// Implicit properties defined by the Urd runtime, not by user type definitions.
/// These are valid in conditions, effects, and rule where clauses on any entity.
/// See Schema Spec §Containment Model.
const IMPLICIT_PROPERTIES: &[&str] = &["container"];

/// Built-in jump targets recognized by the compiler.
/// These are documented in the Schema Markdown spec §Jumps.
const BUILTIN_JUMP_TARGETS: &[&str] = &["end"];

/// Resolve a jump target using the normative priority rule.
fn resolve_jump(
    jump: &mut crate::ast::Jump,
    _file_path: &str,
    ctx: &FileContext,
    current_location_id: &Option<String>,
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    // Built-in terminals — check before section/exit lookup.
    if BUILTIN_JUMP_TARGETS.contains(&jump.target.as_str()) {
        // Warn if a user-defined section shadows this built-in.
        if ctx.local_sections.contains_key(&jump.target) {
            diagnostics.warning(
                "URD431",
                format!(
                    "Section '{}' shadows the built-in '-> {}' terminal. \
                     The jump will always end the conversation, not jump to this section. \
                     Consider renaming the section.",
                    jump.target, jump.target,
                ),
                jump.span.clone(),
            );
        }
        jump.annotation = Some(Annotation {
            ..Default::default()
        });
        return;
    }

    if jump.is_exit_qualified {
        // Explicit exit jump: -> exit:name
        if current_location_id.is_none() {
            diagnostics.error(
                "URD314",
                "Exit construct outside of a location context.",
                jump.span.clone(),
            );
            return;
        }
        let loc_id = current_location_id.as_ref().unwrap();
        if let Some(loc_sym) = symbol_table.locations.get(loc_id) {
            if loc_sym.exits.contains_key(&jump.target) {
                jump.annotation = Some(Annotation {
                    resolved_location: Some(loc_id.clone()),
                    ..Default::default()
                });
            } else {
                diagnostics.error(
                    "URD311",
                    format!(
                        "Unresolved exit reference 'exit:{}'. No exit with this name exists in the current location.",
                        jump.target,
                    ),
                    jump.span.clone(),
                );
            }
        }
        return;
    }

    // Standard jump: -> name
    // Priority: section first, exit second, error third.
    let section_match = ctx.local_sections.get(&jump.target);
    let exit_match = current_location_id.as_ref().and_then(|loc_id| {
        symbol_table
            .locations
            .get(loc_id)
            .and_then(|loc| loc.exits.get(&jump.target))
    });

    match (section_match, exit_match) {
        (Some(compiled_id), Some(_)) => {
            // Section wins, but warn about shadowing.
            jump.annotation = Some(Annotation {
                resolved_section: Some(compiled_id.clone()),
                ..Default::default()
            });
            diagnostics.warning(
                "URD310",
                format!(
                    "Section '{}' shadows exit '{}' in this location. Use -> exit:{} to target the exit.",
                    jump.target, jump.target, jump.target,
                ),
                jump.span.clone(),
            );
        }
        (Some(compiled_id), None) => {
            jump.annotation = Some(Annotation {
                resolved_section: Some(compiled_id.clone()),
                ..Default::default()
            });
        }
        (None, Some(_exit_sym)) => {
            let loc_id = current_location_id.as_ref().unwrap();
            jump.annotation = Some(Annotation {
                resolved_location: Some(loc_id.clone()),
                ..Default::default()
            });
        }
        (None, None) => {
            diagnostics.error(
                "URD309",
                format!(
                    "Unresolved jump target '{}'. No section or exit with this name exists in scope.",
                    jump.target,
                ),
                jump.span.clone(),
            );
        }
    }
}
