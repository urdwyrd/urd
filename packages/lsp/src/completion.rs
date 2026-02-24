/// Autocomplete handler — provides context-aware completion lists.

use lsp_server::Connection;
use lsp_types::*;

use crate::world_state::{self, WorldState};
use urd_compiler::definition_index::DefinitionKind;

pub fn handle(connection: &Connection, state: &WorldState, req: lsp_server::Request) {
    let params: CompletionParams = serde_json::from_value(req.params.clone()).unwrap();

    let items = build_completions(state, &params);

    let result = CompletionResponse::Array(items);
    let response = lsp_server::Response::new_ok(req.id, result);
    connection
        .sender
        .send(lsp_server::Message::Response(response))
        .ok();
}

fn build_completions(state: &WorldState, params: &CompletionParams) -> Vec<CompletionItem> {
    let uri = &params.text_document_position.text_document.uri;
    let position = &params.text_document_position.position;

    // Read the source line to determine context
    let path = world_state::uri_to_path(uri);
    let source = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    let line = match source.lines().nth(position.line as usize) {
        Some(l) => l,
        None => return vec![],
    };

    let col = position.character as usize;
    let before_cursor = if col <= line.len() { &line[..col] } else { line };

    // Determine trigger context
    let trimmed = before_cursor.trim_end();

    // After "@entity." — complete properties for the entity's type
    if let Some(entity_prop_prefix) = extract_entity_dot_prefix(trimmed) {
        return complete_entity_properties(state, &entity_prop_prefix);
    }

    // After "@" — complete entity IDs
    if trimmed.ends_with('@') || (trimmed.contains('@') && !trimmed.contains('.')) {
        return complete_entities(state);
    }

    // After "-> " — complete section names
    if trimmed.ends_with("-> ") || trimmed.ends_with("->") {
        return complete_sections(state);
    }

    // After "->" followed by partial text
    if let Some(arrow_pos) = trimmed.rfind("-> ") {
        let partial = &trimmed[arrow_pos + 3..];
        if !partial.contains(' ') {
            return complete_sections(state);
        }
    }

    vec![]
}

/// Extract the entity ID from a pattern like `@entity_id.`
fn extract_entity_dot_prefix(text: &str) -> Option<String> {
    // Find the last '@' and check if there's a '.' after it
    let at_pos = text.rfind('@')?;
    let after_at = &text[at_pos + 1..];

    if let Some(dot_pos) = after_at.rfind('.') {
        let entity_id = &after_at[..dot_pos];
        if !entity_id.is_empty() && entity_id.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Some(entity_id.to_string());
        }
    }

    None
}

/// Complete entity IDs from the compiled world JSON.
fn complete_entities(state: &WorldState) -> Vec<CompletionItem> {
    let world = match &state.world_json {
        Some(w) => w,
        None => return vec![],
    };

    let entities = match world["entities"].as_object() {
        Some(e) => e,
        None => return vec![],
    };

    entities
        .iter()
        .map(|(id, entity)| {
            let type_name = entity["type"].as_str().unwrap_or("").to_string();
            CompletionItem {
                label: format!("@{}", id),
                kind: Some(CompletionItemKind::VARIABLE),
                detail: Some(type_name),
                insert_text: Some(id.clone()),
                ..Default::default()
            }
        })
        .collect()
}

/// Complete properties for an entity's type.
fn complete_entity_properties(state: &WorldState, entity_id: &str) -> Vec<CompletionItem> {
    let world = match &state.world_json {
        Some(w) => w,
        None => return vec![],
    };

    // Resolve entity to type
    let type_name = match world["entities"][entity_id]["type"].as_str() {
        Some(t) => t,
        None => return vec![],
    };

    // Get type's properties from type definitions
    let properties = match world["types"][type_name]["properties"].as_object() {
        Some(p) => p,
        None => return vec![],
    };

    properties
        .iter()
        .map(|(name, prop)| {
            let prop_type = prop["type"].as_str().unwrap_or("").to_string();
            CompletionItem {
                label: name.clone(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some(prop_type),
                ..Default::default()
            }
        })
        .collect()
}

/// Complete section names from the DefinitionIndex.
fn complete_sections(state: &WorldState) -> Vec<CompletionItem> {
    let index = match &state.definition_index {
        Some(i) => i,
        None => return vec![],
    };

    // Collect unique local_names from sections
    let mut seen = std::collections::HashSet::new();
    index
        .iter()
        .filter_map(|(_, entry)| match &entry.kind {
            DefinitionKind::Section {
                local_name,
                file_stem,
            } => {
                if seen.insert(local_name.clone()) {
                    Some(CompletionItem {
                        label: local_name.clone(),
                        kind: Some(CompletionItemKind::REFERENCE),
                        detail: Some(format!("in {}", file_stem)),
                        ..Default::default()
                    })
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect()
}
