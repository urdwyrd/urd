/// Pass 1: Collection — walk every AST and register declarations.
///
/// Processes files in topological order (`ordered_asts`). For each file:
/// 1. Compute `FileContext` (file stem, visible scope, local sections).
/// 2. Walk frontmatter: register types, entities; store world config.
/// 3. Walk content: register locations, sections, choices, exits, sequences, rules.

use indexmap::IndexMap;

use crate::ast::{
    Choice, ContentNode, FrontmatterValue, LocationHeading, PhaseHeading, RuleBlock, SectionLabel,
    SequenceHeading,
};
use crate::diagnostics::DiagnosticCollector;
use crate::graph::{file_stem, DependencyGraph};
use crate::slugify::slugify;
use crate::span::FilePath;
use crate::symbol_table::{
    ActionSymbol, AstNodeRef, ChoiceSymbol, Duplicate, ExitSymbol, LocationSymbol, PhaseSymbol,
    PropertySymbol, SectionSymbol, SelectDef, SequenceSymbol, SymbolTable, TypeSymbol,
    EntitySymbol, RuleSymbol,
};

use super::{parse_property_type, scalar_to_value, visible_scope, FileContext, WorldConfig};

/// Run collection pass over all files.
pub(crate) fn collect(
    graph: &DependencyGraph,
    ordered_asts: &[FilePath],
    symbol_table: &mut SymbolTable,
    world_config: &mut WorldConfig,
    file_contexts: &mut IndexMap<String, FileContext>,
    diagnostics: &mut DiagnosticCollector,
) {
    for file_path in ordered_asts {
        let node = match graph.nodes.get(file_path.as_str()) {
            Some(n) => n,
            None => continue,
        };

        let stem = file_stem(file_path);
        let scope = visible_scope(file_path, graph);

        let mut ctx = FileContext {
            file_stem: stem.clone(),
            visible_scope: scope,
            local_sections: IndexMap::new(),
        };

        // Tracking state for the content walk.
        let mut current_location_id: Option<String> = None;
        let mut current_section_id: Option<String> = None;
        let mut current_sequence_id: Option<String> = None;

        // Walk frontmatter.
        if let Some(fm) = &node.ast.frontmatter {
            for entry in &fm.entries {
                collect_frontmatter_entry(
                    &entry.value,
                    &stem,
                    file_path,
                    symbol_table,
                    world_config,
                    diagnostics,
                );
            }
        }

        // Walk content.
        for (node_index, content) in node.ast.content.iter().enumerate() {
            collect_content_node(
                content,
                node_index,
                file_path,
                &stem,
                &mut ctx,
                &mut current_location_id,
                &mut current_section_id,
                &mut current_sequence_id,
                symbol_table,
                diagnostics,
            );
        }

        file_contexts.insert(file_path.clone(), ctx);
    }
}

/// Process a single frontmatter entry during collection.
fn collect_frontmatter_entry(
    value: &FrontmatterValue,
    _file_stem: &str,
    _file_path: &str,
    symbol_table: &mut SymbolTable,
    world_config: &mut WorldConfig,
    diagnostics: &mut DiagnosticCollector,
) {
    match value {
        FrontmatterValue::TypeDef(td) => {
            let mut properties = IndexMap::new();
            for prop in &td.properties {
                let prop_type = parse_property_type(&prop.property_type);
                let default = prop.default.as_ref().map(|s| scalar_to_value(s));
                let visibility = match prop.visibility.as_deref() {
                    Some("hidden") => crate::symbol_table::Visibility::Hidden,
                    _ => crate::symbol_table::Visibility::Visible,
                };

                let element_type = prop.element_type.as_ref().map(|s| parse_property_type(s));

                properties.insert(
                    prop.name.clone(),
                    PropertySymbol {
                        name: prop.name.clone(),
                        property_type: prop_type,
                        default,
                        visibility,
                        values: prop.values.clone(),
                        min: prop.min,
                        max: prop.max,
                        ref_type: prop.ref_type.clone(),
                        element_type,
                        element_values: prop.element_values.clone(),
                        element_ref_type: prop.element_ref_type.clone(),
                        description: prop.description.clone(),
                        declared_in: prop.span.clone(),
                    },
                );
            }

            let type_sym = TypeSymbol {
                name: td.name.clone(),
                traits: td.traits.clone(),
                properties,
                declared_in: td.span.clone(),
            };

            if symbol_table.types.contains_key(&td.name) {
                let first = &symbol_table.types[&td.name];
                diagnostics.error(
                    "URD303",
                    format!(
                        "Duplicate type name '{}' declared in {}:{} and {}:{}.",
                        td.name,
                        first.declared_in.file,
                        first.declared_in.start_line,
                        td.span.file,
                        td.span.start_line,
                    ),
                    td.span.clone(),
                );
                symbol_table.duplicates.push(Duplicate {
                    namespace: "types",
                    name: td.name.clone(),
                    declared_in: td.span.clone(),
                });
            } else {
                symbol_table.types.insert(td.name.clone(), type_sym);
            }
        }

        FrontmatterValue::EntityDecl(ed) => {
            let mut overrides = IndexMap::new();
            for (key, val) in &ed.property_overrides {
                overrides.insert(key.clone(), scalar_to_value(val));
            }

            let entity_sym = EntitySymbol {
                id: ed.id.clone(),
                type_name: ed.type_name.clone(),
                type_symbol: None,
                property_overrides: overrides,
                declared_in: ed.span.clone(),
            };

            if symbol_table.entities.contains_key(&ed.id) {
                let first = &symbol_table.entities[&ed.id];
                diagnostics.error(
                    "URD302",
                    format!(
                        "Duplicate entity ID '@{}' declared in {}:{} and {}:{}.",
                        ed.id,
                        first.declared_in.file,
                        first.declared_in.start_line,
                        ed.span.file,
                        ed.span.start_line,
                    ),
                    ed.span.clone(),
                );
                symbol_table.duplicates.push(Duplicate {
                    namespace: "entities",
                    name: ed.id.clone(),
                    declared_in: ed.span.clone(),
                });
            } else {
                symbol_table.entities.insert(ed.id.clone(), entity_sym);
            }
        }

        FrontmatterValue::WorldBlock(wb) => {
            for (key, val) in &wb.fields {
                match key.as_str() {
                    "start" => {
                        if let crate::ast::Scalar::String(s) = val {
                            *world_config = WorldConfig {
                                start: Some((s.clone(), wb.span.clone())),
                                ..std::mem::take(world_config)
                            };
                        }
                    }
                    "entry" => {
                        if let crate::ast::Scalar::String(s) = val {
                            world_config.entry = Some((s.clone(), wb.span.clone()));
                        }
                    }
                    _ => {}
                }
            }
        }

        FrontmatterValue::ImportDecl(_) => {
            // Already processed by IMPORT.
        }

        // Map values (from `types:` / `entities:` blocks) — recurse into entries.
        FrontmatterValue::Map(entries) => {
            for entry in entries {
                collect_frontmatter_entry(
                    &entry.value,
                    _file_stem,
                    _file_path,
                    symbol_table,
                    world_config,
                    diagnostics,
                );
            }
        }

        // Other frontmatter values (lists, scalars) are not declarations.
        _ => {}
    }
}

/// Process a single content node during collection.
fn collect_content_node(
    node: &ContentNode,
    _node_index: usize,
    file_path: &str,
    file_stem: &str,
    ctx: &mut FileContext,
    current_location_id: &mut Option<String>,
    current_section_id: &mut Option<String>,
    current_sequence_id: &mut Option<String>,
    symbol_table: &mut SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    match node {
        ContentNode::LocationHeading(loc) => {
            collect_location(loc, file_path, current_location_id, current_section_id, symbol_table, diagnostics);
        }

        ContentNode::SectionLabel(sec) => {
            collect_section(sec, file_path, file_stem, ctx, current_section_id, symbol_table, diagnostics);
        }

        ContentNode::SequenceHeading(seq) => {
            collect_sequence(seq, current_sequence_id, symbol_table, diagnostics);
        }

        ContentNode::PhaseHeading(phase) => {
            collect_phase(phase, current_sequence_id, symbol_table, diagnostics);
        }

        ContentNode::Choice(choice) => {
            collect_choice(choice, file_path, current_section_id, symbol_table, diagnostics);
        }

        ContentNode::ExitDeclaration(exit) => {
            if current_location_id.is_none() {
                diagnostics.error(
                    "URD314",
                    "Exit construct outside of a location context.",
                    exit.span.clone(),
                );
                return;
            }
            let loc_id = current_location_id.as_ref().unwrap();
            if let Some(loc_sym) = symbol_table.locations.get_mut(loc_id) {
                let condition_node = exit
                    .children
                    .iter()
                    .enumerate()
                    .find(|(_, c)| matches!(c, ContentNode::Condition(_)))
                    .map(|(i, _)| AstNodeRef {
                        file: file_path.to_string(),
                        node_index: i,
                    });
                let blocked_message_node = exit
                    .children
                    .iter()
                    .enumerate()
                    .find(|(_, c)| matches!(c, ContentNode::BlockedMessage(_)))
                    .map(|(i, _)| AstNodeRef {
                        file: file_path.to_string(),
                        node_index: i,
                    });

                let exit_sym = ExitSymbol {
                    direction: exit.direction.clone(),
                    destination: exit.destination.clone(),
                    resolved_destination: None,
                    condition_node,
                    blocked_message_node,
                    declared_in: exit.span.clone(),
                };
                loc_sym.exits.insert(exit.direction.clone(), exit_sym);
            }
        }

        ContentNode::EntityPresence(ep) => {
            if current_location_id.is_none() {
                diagnostics.error(
                    "URD314",
                    "Entity presence construct outside of a location context.",
                    ep.span.clone(),
                );
                return;
            }
            // Raw refs stored; resolution in pass 2.
        }

        ContentNode::RuleBlock(rule) => {
            collect_rule(rule, symbol_table, diagnostics);
        }

        ContentNode::ErrorNode(_) => {
            // Skip silently.
        }

        // All other content nodes contain references, not declarations.
        // Processed in pass 2.
        _ => {}
    }
}

fn collect_location(
    loc: &LocationHeading,
    _file_path: &str,
    current_location_id: &mut Option<String>,
    current_section_id: &mut Option<String>,
    symbol_table: &mut SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    let id = slugify(&loc.display_name);
    if id.is_empty() {
        diagnostics.error(
            "URD313",
            format!(
                "Heading '{}' produces an empty ID after slugification.",
                loc.display_name,
            ),
            loc.span.clone(),
        );
        return;
    }

    if symbol_table.locations.contains_key(&id) {
        let first = &symbol_table.locations[&id];
        diagnostics.error(
            "URD304",
            format!(
                "Duplicate location ID '{}' — locations '{}' and '{}' both slugify to '{}'.",
                id,
                first.display_name,
                loc.display_name,
                id,
            ),
            loc.span.clone(),
        );
        symbol_table.duplicates.push(Duplicate {
            namespace: "locations",
            name: id.clone(),
            declared_in: loc.span.clone(),
        });
    } else {
        symbol_table.locations.insert(
            id.clone(),
            LocationSymbol {
                id: id.clone(),
                display_name: loc.display_name.clone(),
                exits: IndexMap::new(),
                contains: Vec::new(),
                declared_in: loc.span.clone(),
            },
        );
    }

    *current_location_id = Some(id);
    // Sections reset at location boundaries is not required by the brief,
    // but section context persists across locations within a file.
    *current_section_id = None;
}

fn collect_section(
    sec: &SectionLabel,
    file_path: &str,
    file_stem: &str,
    ctx: &mut FileContext,
    current_section_id: &mut Option<String>,
    symbol_table: &mut SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    let compiled_id = format!("{}/{}", file_stem, sec.name);

    // Check for duplicate local name within the file.
    if ctx.local_sections.contains_key(&sec.name) {
        diagnostics.error(
            "URD305",
            format!(
                "Duplicate section name '{}' in {}. Section names must be unique within a file.",
                sec.name,
                file_path,
            ),
            sec.span.clone(),
        );
        symbol_table.duplicates.push(Duplicate {
            namespace: "sections",
            name: compiled_id.clone(),
            declared_in: sec.span.clone(),
        });
    } else {
        ctx.local_sections.insert(sec.name.clone(), compiled_id.clone());

        let section_sym = SectionSymbol {
            local_name: sec.name.clone(),
            compiled_id: compiled_id.clone(),
            file_stem: file_stem.to_string(),
            choices: Vec::new(),
            declared_in: sec.span.clone(),
        };
        symbol_table.sections.insert(compiled_id.clone(), section_sym);
    }

    *current_section_id = Some(compiled_id);
}

fn collect_sequence(
    seq: &SequenceHeading,
    current_sequence_id: &mut Option<String>,
    symbol_table: &mut SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    let id = slugify(&seq.display_name);
    if id.is_empty() {
        diagnostics.error(
            "URD313",
            format!(
                "Heading '{}' produces an empty ID after slugification.",
                seq.display_name,
            ),
            seq.span.clone(),
        );
        return;
    }

    if symbol_table.sequences.contains_key(&id) {
        let first = &symbol_table.sequences[&id];
        diagnostics.error(
            "URD313",
            format!(
                "Duplicate sequence ID '{}' — sequences '{}' and '{}' both slugify to '{}'.",
                id,
                first.id,
                seq.display_name,
                id,
            ),
            seq.span.clone(),
        );
        symbol_table.duplicates.push(Duplicate {
            namespace: "sequences",
            name: id.clone(),
            declared_in: seq.span.clone(),
        });
    } else {
        let seq_sym = SequenceSymbol {
            id: id.clone(),
            phases: Vec::new(),
            declared_in: seq.span.clone(),
        };
        symbol_table.sequences.insert(id.clone(), seq_sym);
    }
    *current_sequence_id = Some(id);
}

fn collect_phase(
    phase: &PhaseHeading,
    current_sequence_id: &mut Option<String>,
    symbol_table: &mut SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    let id = slugify(&phase.display_name);
    if id.is_empty() {
        diagnostics.error(
            "URD313",
            format!(
                "Heading '{}' produces an empty ID after slugification.",
                phase.display_name,
            ),
            phase.span.clone(),
        );
        return;
    }

    if let Some(seq_id) = current_sequence_id {
        if let Some(seq_sym) = symbol_table.sequences.get_mut(seq_id) {
            let phase_sym = PhaseSymbol {
                id,
                advance: if phase.auto { "auto".to_string() } else { "manual".to_string() },
                action: None,
                actions: None,
                rule: None,
                declared_in: phase.span.clone(),
            };
            seq_sym.phases.push(phase_sym);
        }
    }
}

fn collect_choice(
    choice: &Choice,
    file_path: &str,
    current_section_id: &mut Option<String>,
    symbol_table: &mut SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    let section_id = match current_section_id {
        Some(id) => id.clone(),
        None => return, // No section context — skip.
    };

    let slug = slugify(&choice.label);
    if slug.is_empty() {
        diagnostics.error(
            "URD313",
            format!(
                "Heading '{}' produces an empty ID after slugification.",
                choice.label,
            ),
            choice.span.clone(),
        );
        return;
    }

    let choice_id = format!("{}/{}", section_id, slug);

    // Check for duplicate choice slug within the section.
    if let Some(sec_sym) = symbol_table.sections.get(&section_id) {
        if sec_sym.choices.iter().any(|c| c.compiled_id == choice_id) {
            let first = sec_sym.choices.iter().find(|c| c.compiled_id == choice_id).unwrap();
            diagnostics.error(
                "URD306",
                format!(
                    "Duplicate choice ID '{}' in section '{}'. Choices '{}' and '{}' produce the same slugified ID.",
                    choice_id,
                    section_id,
                    first.label,
                    choice.label,
                ),
                choice.span.clone(),
            );
            symbol_table.duplicates.push(Duplicate {
                namespace: "choices",
                name: choice_id,
                declared_in: choice.span.clone(),
            });
            return;
        }
    }

    let choice_sym = ChoiceSymbol {
        label: choice.label.clone(),
        compiled_id: choice_id.clone(),
        sticky: choice.sticky,
        declared_in: choice.span.clone(),
    };

    // Attach to the section.
    if let Some(sec_sym) = symbol_table.sections.get_mut(&section_id) {
        sec_sym.choices.push(choice_sym);
    }

    // Create corresponding ActionSymbol.
    let action_sym = ActionSymbol {
        id: choice_id.clone(),
        target: choice.target.clone(),
        target_type: choice.target_type.clone(),
        declared_in: choice.span.clone(),
    };

    if symbol_table.actions.contains_key(&choice_id) {
        // Duplicate action — standard first-wins rule.
        symbol_table.duplicates.push(Duplicate {
            namespace: "actions",
            name: choice_id,
            declared_in: choice.span.clone(),
        });
    } else {
        symbol_table.actions.insert(choice_id, action_sym);
    }

    // Recurse into nested choices.
    for child in &choice.content {
        if let ContentNode::Choice(sub_choice) = child {
            collect_choice(sub_choice, file_path, current_section_id, symbol_table, diagnostics);
        }
    }
}

fn collect_rule(
    rule: &RuleBlock,
    symbol_table: &mut SymbolTable,
    diagnostics: &mut DiagnosticCollector,
) {
    let select = rule.select.as_ref().map(|s| SelectDef {
        variable: s.variable.clone(),
        from: s.entity_refs.clone(),
        where_clauses: s.where_clauses.clone(),
        span: s.span.clone(),
    });

    let rule_sym = RuleSymbol {
        id: rule.name.clone(),
        actor: rule.actor.clone(),
        trigger: rule.trigger.clone(),
        select,
        declared_in: rule.span.clone(),
    };

    if symbol_table.rules.contains_key(&rule.name) {
        let first = &symbol_table.rules[&rule.name];
        diagnostics.error(
            "URD302",
            format!(
                "Duplicate rule name '{}' declared in {}:{} and {}:{}.",
                rule.name,
                first.declared_in.file,
                first.declared_in.start_line,
                rule.span.file,
                rule.span.start_line,
            ),
            rule.span.clone(),
        );
        symbol_table.duplicates.push(Duplicate {
            namespace: "rules",
            name: rule.name.clone(),
            declared_in: rule.span.clone(),
        });
    } else {
        symbol_table.rules.insert(rule.name.clone(), rule_sym);
    }
}
